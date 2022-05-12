use crate::result::Result;
use crate::use_cases::bus::{Bus, Event, Subscriber};
use crate::use_cases::config::Config;

use retry::delay::{jitter, Exponential};
use retry::retry;
use std::cell::RefCell;
use std::fmt::Debug;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{debug, info, instrument};
use tungstenite::error::ProtocolError;
use tungstenite::{accept, Error, Message, WebSocket};

/// Accepts new websocket connections, notifies all connected clients and performs socket cleanup.
///
/// It performs three jobs:
/// - When client connects to the core, it holds the socket and informs connected client that the
/// connection succeeded.
/// - When [`Event::DocumentReady`] event appears on the bus, it notifies all connected devices, via
/// stored sockets, about new documents ready to be displayed.
/// - When one of the stored socket receives connection closed event or losts connection, this
/// socket is removed from memory. This cleanup is performed periodically and the period of the
/// check is controlled by [`Config::websocket_cleanup_time`].
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
            info!("waiting for a connection...");
            for stream in listener.incoming() {
                let stream = stream?;
                // Initially this was `stream.set_nonblocking(true)`, but it causes some issues
                // with a handshake when clients connect to the core.
                // The timeout (or non-blocking) is needed because of the connection cleanup which
                // needs to read from a sockets to detect closed connection. It needs to iterate
                // through sockets and thus it cannot block forever.
                stream.set_read_timeout(Some(Duration::from_secs(3)))?;
                info!("\tconnection accepted");
                let websocket = accept(stream)?;
                info!("\twebsocket ready");
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
        let cleanup_duration = self.cfg.websocket_cleanup_time;
        thread::spawn(move || -> Result<()> {
            loop {
                let mut sockets = sockets.all.lock().expect("poisoned mutex");
                sockets.retain(Socket::is_active);
                drop(sockets);
                debug!("sleeping for {} seconds", cleanup_duration.as_secs());
                thread::sleep(cleanup_duration);
            }
        });
    }
}

#[derive(Clone)]
struct NotifiableSockets {
    all: Arc<Mutex<Vec<Socket>>>,
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

    fn add(&mut self, socket: Socket) {
        info!("adding socket {:?}", socket);
        let mut all = self.all.lock().expect("poisoned mutex");
        all.push(socket);
        debug!("number of sockets connected: {}", all.len());
    }

    fn await_notifications(&self, sub: Box<dyn Subscriber>) {
        info!("awaiting notifications");
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
    websocket: RefCell<WebSocket<TcpStream>>,
}

impl Socket {
    fn new(websocket: WebSocket<TcpStream>) -> Self {
        Self {
            websocket: RefCell::new(websocket),
        }
    }

    fn inform_connected(&mut self) -> Result<()> {
        info!("notifying about connection established...");
        self.websocket
            .borrow_mut()
            .write_message(Message::Text("connected".into()))?;
        debug!("notified");
        Ok(())
    }

    fn notify_new_docs(&mut self) -> Result<()> {
        info!("notifying about new docs...");
        retry(Exponential::from_millis(200).map(jitter).take(3), || {
            self.websocket
                .borrow_mut()
                .write_message(Message::Text("new-doc".into()))
        })?;
        debug!("notified");
        Ok(())
    }

    fn is_active(&self) -> bool {
        debug!("checking if socket {:?} is active", self);
        match self.websocket.borrow_mut().read_message() {
            Ok(Message::Close(_))
            | Err(
                Error::ConnectionClosed
                | Error::AlreadyClosed
                | Error::Protocol(ProtocolError::ResetWithoutClosingHandshake),
            ) => {
                info!("connection closed");
                false
            }
            _e => {
                debug!("other event: {:?}", _e);
                true
            }
        }
    }
}

impl Debug for Socket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.websocket
                .borrow()
                .get_ref()
                .peer_addr()
                .expect("failed to get addr")
                .to_string()
        )
    }
}
