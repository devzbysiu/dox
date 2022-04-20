use crate::cfg::Config;
use crate::result::Result;
use crate::use_cases::notifier::Notifier;

use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::{accept, Message, WebSocket};

use tracing::debug;

pub struct WsNotifier {
    tx: Sender<()>,
}

impl WsNotifier {
    pub fn new(cfg: &Config) -> Result<Self> {
        let (tx, rx) = channel();
        let (sockets_list, tx) = NotifiableSockets::new(tx);
        sockets_list.await_notifications(rx);
        ConnHandler::new(cfg)?.push_new_conns(sockets_list);
        Ok(Self { tx })
    }
}

impl Notifier for WsNotifier {
    fn notify(&self) -> Result<()> {
        debug!("notifying all listeners");
        self.tx.send(())?;
        Ok(())
    }
}

struct ConnHandler {
    listener: TcpListener,
}

impl ConnHandler {
    fn new(cfg: &Config) -> Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(&cfg.notifications_addr)?,
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
    fn new(tx: Sender<()>) -> (Self, Sender<()>) {
        (
            Self {
                all: Arc::new(Mutex::new(Vec::new())),
            },
            tx,
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
