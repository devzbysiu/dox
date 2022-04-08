use crate::result::Result;

use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::{accept, Message, WebSocket};

use log::debug;

#[allow(clippy::module_name_repetitions)]
pub fn new_doc_notifier() -> Result<Notifier> {
    debug!("creating new doc notifier");
    let (tx, rx) = channel();
    let (sockets_list, notifier) = NotifiableSockets::new(tx);
    debug!("before awaiting notifications");
    sockets_list.await_notifications(rx);
    debug!("creating new connection handler");
    ConnHandler::new()?.push_new_conns(sockets_list);
    debug!("returning notifier");
    Ok(notifier)
}

struct ConnHandler {
    listener: TcpListener,
}

impl ConnHandler {
    fn new() -> Result<Self> {
        debug!("in connhandler constructor");
        // TODO: DOX_WEBSOCKET_ADDR should be passed by config and overwritten in
        // main - see DOX_CONFIG_PATH
        let addr = std::env::var("DOX_WEBSOCKET_ADDR")
            .ok()
            .unwrap_or("0.0.0.0:8001".to_string());
        let handler = Self {
            listener: TcpListener::bind(addr)?,
        };
        debug!("in connhandler constructor");
        Ok(handler)
    }

    fn push_new_conns(self, mut sockets: NotifiableSockets) {
        thread::spawn(move || -> Result<()> {
            debug!("waiting for a connection...");
            for stream in self.listener.incoming() {
                let stream = stream?;
                debug!("\tconnection accepted");
                let websocket = accept(stream)?;
                debug!("\twebsocket ready");
                let mut socket = Socket::new(websocket);
                socket.inform_connected()?;
                sockets.add(socket);
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
        debug!("awaiting notifications");
        let all = self.all.clone();
        thread::spawn(move || -> Result<()> {
            loop {
                rx.recv()?;
                let _errors = all // TODO: take care of that
                    .lock()
                    .expect("poisoned mutex")
                    .iter_mut()
                    .map(Socket::notify_new_docs)
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

    fn inform_connected(&mut self) -> Result<()> {
        debug!("notifying about connection established...");
        self.websocket
            .write_message(Message::Text("connected".into()))?;
        debug!("notified");
        Ok(())
    }

    fn notify_new_docs(&mut self) -> Result<()> {
        debug!("notifying about new docs...");
        self.websocket
            .write_message(Message::Text("new-doc".into()))?;
        debug!("notified");
        Ok(())
    }
}
