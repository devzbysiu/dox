use crate::result::Result;
use crate::use_cases::bus::{Bus, Event, Subscriber};
use crate::use_cases::config::Config;

use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::{accept, Message, WebSocket};

use tracing::debug;

/// Accepts new WebSocket connections and notifies all connected parties.
///
/// When [`Event::DocumentReady`] event appears on the bus, it notifies all connected devices about
/// new documents, ready to be displayed.
#[allow(clippy::module_name_repetitions)]
pub struct WsNotifier;

impl WsNotifier {
    pub fn run(cfg: &Config, bus: &dyn Bus) -> Result<()> {
        let sub = bus.subscriber();
        let sockets_list = NotifiableSockets::new();
        sockets_list.await_notifications(sub);
        ConnHandler::new(cfg)?.push_new_conns(sockets_list);
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
    fn new() -> Self {
        Self {
            all: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add(&mut self, notifier: Socket) {
        debug!("adding socket");
        self.all.lock().expect("poisoned mutex").push(notifier);
    }

    fn await_notifications(&self, sub: Box<dyn Subscriber>) {
        debug!("awaiting notifications");
        let all = self.all.clone();
        thread::spawn(move || -> Result<()> {
            loop {
                match sub.recv()? {
                    Event::DocumentReady => {
                        let _errors = all // TODO: take care of that
                            .lock()
                            .expect("poisoned mutex")
                            .iter_mut()
                            .map(Socket::notify_new_docs)
                            .collect::<Vec<_>>();
                    }
                    _ => debug!("event not supported here"),
                }
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
