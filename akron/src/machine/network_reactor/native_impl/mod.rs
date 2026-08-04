use std::collections::HashMap;
use std::io;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;

use memory::ValueResourceKind;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token, Waker};

use crate::native::NativeAsyncOutput;

use super::*;

mod client;
mod io_ops;

pub(crate) use client::NetworkReactor;

const WAKE_TOKEN: Token = Token(0);
const COMMAND_CAPACITY: usize = 256;
const POLL_INTERVAL: Duration = Duration::from_millis(50);

enum Command {
    Submit {
        request: NativeNetworkRequest,
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    },
    Close(u32),
    Shutdown,
}

pub(super) enum Pending {
    Connect {
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    },
    Accept {
        connection: u32,
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    },
    Read {
        max_bytes: usize,
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    },
    Write {
        bytes: Vec<u8>,
        written: usize,
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    },
}

impl Pending {
    pub(super) fn cancelled(&self) -> bool {
        match self {
            Self::Connect { cancelled, .. }
            | Self::Accept { cancelled, .. }
            | Self::Read { cancelled, .. }
            | Self::Write { cancelled, .. } => cancelled.load(Ordering::Acquire),
        }
    }

    pub(super) fn fail(self, message: impl Into<String>) {
        let result = match self {
            Self::Connect { result, .. }
            | Self::Accept { result, .. }
            | Self::Read { result, .. }
            | Self::Write { result, .. } => result,
        };
        let _ = result.send(Err(message.into()));
    }
}

pub(super) struct ReactorLoop {
    pub(super) poll: Poll,
    events: Events,
    commands: mpsc::Receiver<Command>,
    pub(super) listeners: HashMap<u32, TcpListener>,
    pub(super) connections: HashMap<u32, TcpStream>,
    pub(super) pending: HashMap<u32, Pending>,
}

impl ReactorLoop {
    fn run(mut self) {
        loop {
            match self.drain_commands() {
                Ok(true) => return,
                Ok(false) => {}
                Err(error) => {
                    self.fail_all(format!("network reactor command failed: {error}"));
                    return;
                }
            }
            self.cancel_requested();
            if let Err(error) = self.poll.poll(&mut self.events, Some(POLL_INTERVAL)) {
                self.fail_all(format!("network reactor poll failed: {error}"));
                return;
            }
            let ready = self
                .events
                .iter()
                .filter_map(|event| {
                    (event.token() != WAKE_TOKEN).then_some((
                        event.token(),
                        event.is_readable(),
                        event.is_writable(),
                        event.is_error() || event.is_read_closed() || event.is_write_closed(),
                    ))
                })
                .collect::<Vec<_>>();
            for (token, readable, writable, closed) in ready {
                self.handle_ready(token_to_handle(token), readable, writable, closed);
            }
        }
    }

    fn drain_commands(&mut self) -> io::Result<bool> {
        loop {
            match self.commands.try_recv() {
                Ok(Command::Submit {
                    request,
                    cancelled,
                    result,
                }) => self.submit(request, cancelled, result),
                Ok(Command::Close(handle)) => self.close(handle),
                Ok(Command::Shutdown) => return Ok(true),
                Err(mpsc::TryRecvError::Empty) => return Ok(false),
                Err(mpsc::TryRecvError::Disconnected) => return Ok(true),
            }
        }
    }

    fn submit(
        &mut self,
        request: NativeNetworkRequest,
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    ) {
        if cancelled.load(Ordering::Acquire) {
            let _ = result.send(Err("network task cancelled before start".into()));
            return;
        }
        let outcome = match request {
            NativeNetworkRequest::Connect { handle, address } => {
                self.start_connect(handle, address, cancelled, result)
            }
            NativeNetworkRequest::Listen { handle, address } => {
                self.start_listen(handle, address, result)
            }
            NativeNetworkRequest::Accept {
                listener,
                connection,
            } => self.start_accept(listener, connection, cancelled, result),
            NativeNetworkRequest::Read { handle, max_bytes } => {
                self.start_read(handle, max_bytes, cancelled, result)
            }
            NativeNetworkRequest::Write { handle, bytes } => {
                self.start_write(handle, bytes, cancelled, result)
            }
            NativeNetworkRequest::Close { handle } => {
                self.close(handle);
                let _ = result.send(Ok(NativeAsyncOutput::Nil));
                Ok(())
            }
        };
        if let Err((error, result)) = outcome {
            let _ = result.send(Err(error));
        }
    }

    fn start_connect(
        &mut self,
        handle: u32,
        address: std::net::SocketAddr,
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    ) -> StartResult {
        if self.connections.contains_key(&handle) || self.pending.contains_key(&handle) {
            return Err(("connection resource is already active".into(), result));
        }
        let mut stream = TcpStream::connect(address).map_err(|error| {
            (
                format!("tcp_connect({address}) failed: {error}"),
                result.clone(),
            )
        })?;
        self.poll
            .registry()
            .register(&mut stream, handle_to_token(handle), Interest::WRITABLE)
            .map_err(|error| {
                (
                    format!("register tcp connect failed: {error}"),
                    result.clone(),
                )
            })?;
        self.connections.insert(handle, stream);
        self.pending
            .insert(handle, Pending::Connect { cancelled, result });
        Ok(())
    }

    fn start_listen(
        &mut self,
        handle: u32,
        address: std::net::SocketAddr,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    ) -> StartResult {
        if self.listeners.contains_key(&handle) {
            return Err(("listener resource is already active".into(), result));
        }
        let listener = TcpListener::bind(address).map_err(|error| {
            (
                format!("tcp_listen({address}) failed: {error}"),
                result.clone(),
            )
        })?;
        self.listeners.insert(handle, listener);
        let _ = result.send(Ok(NativeAsyncOutput::Resource {
            kind: ValueResourceKind::Listener,
            handle,
        }));
        Ok(())
    }

    fn start_accept(
        &mut self,
        listener: u32,
        connection: u32,
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    ) -> StartResult {
        if self.pending.contains_key(&listener) {
            return Err(("listener already has a pending accept".into(), result));
        }
        let socket = self
            .listeners
            .get_mut(&listener)
            .ok_or_else(|| ("listener is closed".into(), result.clone()))?;
        self.poll
            .registry()
            .register(socket, handle_to_token(listener), Interest::READABLE)
            .map_err(|error| {
                (
                    format!("register tcp accept failed: {error}"),
                    result.clone(),
                )
            })?;
        self.pending.insert(
            listener,
            Pending::Accept {
                connection,
                cancelled,
                result,
            },
        );
        Ok(())
    }

    fn start_read(
        &mut self,
        handle: u32,
        max_bytes: usize,
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    ) -> StartResult {
        if self.pending.contains_key(&handle) {
            return Err(("connection already has a pending operation".into(), result));
        }
        let socket = self
            .connections
            .get_mut(&handle)
            .ok_or_else(|| ("connection is closed".into(), result.clone()))?;
        self.poll
            .registry()
            .register(socket, handle_to_token(handle), Interest::READABLE)
            .map_err(|error| (format!("register tcp read failed: {error}"), result.clone()))?;
        self.pending.insert(
            handle,
            Pending::Read {
                max_bytes,
                cancelled,
                result,
            },
        );
        Ok(())
    }

    fn start_write(
        &mut self,
        handle: u32,
        bytes: Vec<u8>,
        cancelled: Arc<AtomicBool>,
        result: mpsc::Sender<Result<NativeAsyncOutput, String>>,
    ) -> StartResult {
        if self.pending.contains_key(&handle) {
            return Err(("connection already has a pending operation".into(), result));
        }
        if bytes.is_empty() {
            let _ = result.send(Ok(NativeAsyncOutput::Int(0)));
            return Ok(());
        }
        let socket = self
            .connections
            .get_mut(&handle)
            .ok_or_else(|| ("connection is closed".into(), result.clone()))?;
        self.poll
            .registry()
            .register(socket, handle_to_token(handle), Interest::WRITABLE)
            .map_err(|error| {
                (
                    format!("register tcp write failed: {error}"),
                    result.clone(),
                )
            })?;
        self.pending.insert(
            handle,
            Pending::Write {
                bytes,
                written: 0,
                cancelled,
                result,
            },
        );
        Ok(())
    }
}

type StartResult = Result<(), (String, mpsc::Sender<Result<NativeAsyncOutput, String>>)>;

pub(super) fn handle_to_token(handle: u32) -> Token {
    Token(handle as usize + 1)
}

fn token_to_handle(token: Token) -> u32 {
    (token.0 - 1) as u32
}
