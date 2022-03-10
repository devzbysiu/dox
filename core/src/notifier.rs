use crate::result::Result;

use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use tungstenite::{accept, Message, WebSocket};

use log::debug;

pub fn notifier() -> Result<Notifier> {
    let (notifiers, notifier) = Notifiers::new();
    debug!("creating notifications channel via websocket");
    let server = TcpListener::bind("0.0.0.0:8001")?;
    debug!("waiting for a connection...");
    thread::spawn(|| -> Result<()> {
        for stream in server.incoming() {
            let stream = stream?;
            debug!("stream accepted");
            let websocket = accept(stream)?;
            debug!("websocket ready");
            notifiers.add(NewDocsNotifier::new(websocket));
        }
        Ok(())
    });

    Ok(notifier)
}

struct Notifiers {
    all: Vec<NewDocsNotifier>,
    rx: Receiver<()>,
}

impl Notifiers {
    fn new() -> (Self, Notifier) {
        let (tx, rx) = channel();
        (
            Self {
                all: Vec::new(),
                rx,
            },
            Notifier::new(tx),
        )
    }

    fn add(&mut self, notifier: NewDocsNotifier) {
        self.all.push(notifier);
    }

    fn start_listening(self) {
        let rx = self.rx;
        thread::spawn(move || -> Result<()> {
            loop {
                let _ = rx.recv()?;
                self.all.iter_mut().map(NewDocsNotifier::notify);
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

    pub fn notify(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.tx.send(())?;
        Ok(())
    }
}

struct NewDocsNotifier {
    websocket: WebSocket<TcpStream>,
}

impl NewDocsNotifier {
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
