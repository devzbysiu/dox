use crate::result::Result;
use crate::use_cases::bus::{Bus, Event, Subscriber};
use crate::use_cases::config::Config;

use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tungstenite::error::ProtocolError;
use tungstenite::{accept, Error, Message, WebSocket};

use tracing::{debug, instrument};

/// Accepts new websocket connections and notifies all connected parties.
///
/// When [`Event::DocumentReady`] event appears on the bus, it notifies all connected devices about
/// new documents, ready to be displayed.
pub struct WsNotifier<'a> {
    cfg: &'a Config,
    bus: &'a dyn Bus,
}

impl<'a> WsNotifier<'a> {
    pub fn new(cfg: &'a Config, bus: &'a dyn Bus) -> Self {
        Self { cfg, bus }
    }

    #[instrument(skip(self))]
    pub fn run(&self) -> Result<()> {
        ConnHandler::new(self.cfg.clone(), NotifiableSockets::new(self.bus))?;
        Ok(())
    }
}

struct ConnHandler {
    cfg: Config,
    sockets: NotifiableSockets,
}

impl ConnHandler {
    #[instrument(skip(sockets))]
    fn new(cfg: Config, sockets: NotifiableSockets) -> Result<Self> {
        let handler = Self { cfg, sockets };
        handler.push_new_conns()?;
        handler.run_periodic_cleanup();
        Ok(handler)
    }

    fn push_new_conns(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.cfg.notifications_addr)?;
        let mut sockets = self.sockets.clone();
        thread::spawn(move || -> Result<()> {
            debug!("waiting for a connection...");
            for stream in listener.incoming() {
                let stream = stream?;
                stream.set_nonblocking(true)?;
                debug!("\tconnection accepted");
                let websocket = accept(stream)?;
                debug!("\twebsocket ready");
                let mut socket = Socket::new(websocket);
                socket.inform_connected()?;
                sockets.add(socket);
            }
            Ok(())
        });
        Ok(())
    }

    fn run_periodic_cleanup(&self) {
        let sockets = self.sockets.clone();
        thread::spawn(move || -> Result<()> {
            loop {
                let mut idx = 0;
                let mut all_sockets = sockets.all.lock().expect("poisoned mutex");
                debug!("checking for inactive sockets, #: {}", all_sockets.len());
                while idx < all_sockets.len() {
                    let socket = &mut all_sockets[idx];
                    match socket.websocket.read_message() {
                        Ok(Message::Close(_)) => debug!("got closed message"),
                        Err(Error::ConnectionClosed) => {
                            debug!("connection closed, removing socket");
                            all_sockets.remove(idx);
                            continue;
                        }
                        Err(Error::AlreadyClosed) => {
                            debug!("connection already closed, removing socket");
                            all_sockets.remove(idx);
                            continue;
                        }
                        Err(Error::Protocol(ProtocolError::ResetWithoutClosingHandshake)) => {
                            debug!("connection closed abrubptly, removing socket");
                            all_sockets.remove(idx);
                            continue;
                        }
                        Err(Error::Io(e)) if e.kind() == ErrorKind::WouldBlock => {
                            // no message in non-blocking socket, see [`TcpStream::set_nonblocking`]
                        }
                        _e => {
                            debug!("other message: {:?}", _e);
                            idx += 1;
                        }
                    }
                }
                drop(all_sockets);
                debug!("sleeping for 10 seconds");
                thread::sleep(Duration::from_secs(10)); // TODO: take care of this
            }
        });
    }
}

#[derive(Clone)]
struct NotifiableSockets {
    all: Arc<Mutex<Vec<Socket>>>, // TODO: handle case when socket is disconnected
}

impl NotifiableSockets {
    fn new(bus: &dyn Bus) -> Self {
        let sockets = Self {
            all: Arc::new(Mutex::new(Vec::new())),
        };
        let sub = bus.subscriber();
        sockets.await_notifications(sub);
        sockets
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
                if let Event::DocumentReady = sub.recv()? {
                    let _errors = all // TODO: take care of that
                        .lock()
                        .expect("poisoned mutex")
                        .iter_mut()
                        .map(Socket::notify_new_docs)
                        .collect::<Vec<_>>();
                } else {
                    debug!("event not supported here");
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

    fn is_active(&self) -> bool {
        let res = self.websocket.can_write();
        debug!("is_active: {}", res);
        res
    }
}
