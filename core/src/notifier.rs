use crate::result::Result;

use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::{accept, Message, WebSocket};

use log::debug;

pub fn new_doc_notifier() -> Result<Notifier> {
    debug!("creating new doc notifier");
    let (tx, rx) = channel();
    let (sockets_list, notifier) = NotifiableSockets::new(tx);
    sockets_list.await_notifications(rx);
    ConnHandler::new()?.push_new_conns(sockets_list);
    Ok(notifier)
}

struct ConnHandler {
    listener: TcpListener,
}

impl ConnHandler {
    fn new() -> Result<Self> {
        Ok(Self {
            listener: TcpListener::bind("0.0.0.0:8001")?,
        })
    }

    fn push_new_conns(self, mut sockets: NotifiableSockets) {
        thread::spawn(move || -> Result<()> {
            debug!("waiting for a connection...");
            for stream in self.listener.incoming() {
                let stream = stream?;
                debug!("\tconnection accepted");
                let websocket = accept(stream)?;
                debug!("\twebsocket ready");
                sockets.add(Socket::new(websocket));
            }
            Ok(())
        });
    }
}

struct NotifiableSockets {
    all: Arc<Mutex<Vec<Socket>>>, // TODO: handle case when socket is disconnected
}

impl NotifiableSockets {
    fn new(tx: Sender<()>) -> (Self, Notifier) {
        (
            Self {
                all: Arc::new(Mutex::new(Vec::new())),
            },
            Notifier::new(tx),
        )
    }

    fn add(&mut self, notifier: Socket) {
        debug!("adding socket");
        self.all.lock().expect("poisoned mutex").push(notifier);
    }

    fn await_notifications(&self, rx: Receiver<()>) {
        let all = self.all.clone();
        thread::spawn(move || -> Result<()> {
            loop {
                let _ = rx.recv()?;
                let _ = all
                    .lock()
                    .expect("poisoned mutex")
                    .iter_mut()
                    .map(Socket::notify)
                    .collect::<Vec<_>>();
            }
        });
    }
}

pub struct Notifier {
    tx: Sender<()>,
}

impl Notifier {
    fn new(tx: Sender<()>) -> Self {
        Self { tx }
    }

    pub fn notify(&self) -> Result<()> {
        debug!("notifying all listeners");
        self.tx.send(())?;
        Ok(())
    }
}

struct Socket {
    websocket: WebSocket<TcpStream>,
}

impl Socket {
    fn new(websocket: WebSocket<TcpStream>) -> Self {
        Self { websocket }
    }

    fn notify(&mut self) -> Result<()> {
        debug!("notifying about new docs...");
        self.websocket.write_message(Message::Text("".into()))?;
        debug!("notified");
        Ok(())
    }
}
