use std::io::{self, Read, Write};

use memory::ValueResourceKind;

use crate::native::NativeAsyncOutput;

use super::{Pending, ReactorLoop};

impl ReactorLoop {
    pub(super) fn handle_ready(
        &mut self,
        handle: u32,
        readable: bool,
        writable: bool,
        closed: bool,
    ) {
        let Some(pending) = self.pending.remove(&handle) else {
            return;
        };
        if pending.cancelled() {
            self.cancel_operation(handle, pending);
            return;
        }
        match pending {
            Pending::Connect { result, .. } if writable || closed => {
                let completion = self
                    .connections
                    .get(&handle)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "connection closed"))
                    .and_then(|stream| stream.take_error());
                match completion {
                    Ok(None) => {
                        self.deregister_connection(handle);
                        let _ = result.send(Ok(NativeAsyncOutput::Resource {
                            kind: ValueResourceKind::Connection,
                            handle,
                        }));
                    }
                    Ok(Some(error)) | Err(error) => {
                        self.close(handle);
                        let _ = result.send(Err(format!("tcp connect failed: {error}")));
                    }
                }
            }
            Pending::Accept {
                connection,
                cancelled,
                result,
            } if readable || closed => {
                let accepted = self
                    .listeners
                    .get_mut(&handle)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "listener closed"))
                    .and_then(|listener| listener.accept().map(|(stream, _)| stream));
                match accepted {
                    Ok(stream) => {
                        self.deregister_listener(handle);
                        self.connections.insert(connection, stream);
                        let _ = result.send(Ok(NativeAsyncOutput::Resource {
                            kind: ValueResourceKind::Connection,
                            handle: connection,
                        }));
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        self.pending.insert(
                            handle,
                            Pending::Accept {
                                connection,
                                cancelled,
                                result,
                            },
                        );
                    }
                    Err(error) => {
                        self.deregister_listener(handle);
                        let _ = result.send(Err(format!("tcp accept failed: {error}")));
                    }
                }
            }
            Pending::Read {
                max_bytes,
                cancelled,
                result,
            } if readable || closed => {
                let mut bytes = vec![0; max_bytes];
                let outcome = self
                    .connections
                    .get_mut(&handle)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "connection closed"))
                    .and_then(|stream| stream.read(&mut bytes));
                match outcome {
                    Ok(read) => {
                        bytes.truncate(read);
                        self.deregister_connection(handle);
                        let _ = result.send(Ok(NativeAsyncOutput::Bytes(bytes)));
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        self.pending.insert(
                            handle,
                            Pending::Read {
                                max_bytes,
                                cancelled,
                                result,
                            },
                        );
                    }
                    Err(error) => {
                        self.deregister_connection(handle);
                        let _ = result.send(Err(format!("tcp read failed: {error}")));
                    }
                }
            }
            Pending::Write {
                bytes,
                written,
                cancelled,
                result,
            } if writable || closed => {
                let outcome = self
                    .connections
                    .get_mut(&handle)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "connection closed"))
                    .and_then(|stream| stream.write(&bytes[written..]));
                match outcome {
                    Ok(count) if written + count == bytes.len() => {
                        self.deregister_connection(handle);
                        let _ = result.send(Ok(NativeAsyncOutput::Int(bytes.len() as i64)));
                    }
                    Ok(count) => {
                        self.pending.insert(
                            handle,
                            Pending::Write {
                                bytes,
                                written: written + count,
                                cancelled,
                                result,
                            },
                        );
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        self.pending.insert(
                            handle,
                            Pending::Write {
                                bytes,
                                written,
                                cancelled,
                                result,
                            },
                        );
                    }
                    Err(error) => {
                        self.deregister_connection(handle);
                        let _ = result.send(Err(format!("tcp write failed: {error}")));
                    }
                }
            }
            operation => {
                self.pending.insert(handle, operation);
            }
        }
    }

    pub(super) fn cancel_requested(&mut self) {
        let handles = self
            .pending
            .iter()
            .filter_map(|(handle, pending)| pending.cancelled().then_some(*handle))
            .collect::<Vec<_>>();
        for handle in handles {
            if let Some(pending) = self.pending.remove(&handle) {
                self.cancel_operation(handle, pending);
            }
        }
    }

    fn cancel_operation(&mut self, handle: u32, pending: Pending) {
        if matches!(pending, Pending::Connect { .. }) {
            self.close(handle);
        } else if matches!(pending, Pending::Accept { .. }) {
            self.deregister_listener(handle);
        } else {
            self.deregister_connection(handle);
        }
        pending.fail("network operation cancelled");
    }

    pub(super) fn close(&mut self, handle: u32) {
        if let Some(pending) = self.pending.remove(&handle) {
            pending.fail("network resource closed");
        }
        if let Some(mut listener) = self.listeners.remove(&handle) {
            let _ = self.poll.registry().deregister(&mut listener);
        }
        if let Some(mut stream) = self.connections.remove(&handle) {
            let _ = self.poll.registry().deregister(&mut stream);
        }
    }

    fn deregister_listener(&mut self, handle: u32) {
        if let Some(listener) = self.listeners.get_mut(&handle) {
            let _ = self.poll.registry().deregister(listener);
        }
    }

    fn deregister_connection(&mut self, handle: u32) {
        if let Some(stream) = self.connections.get_mut(&handle) {
            let _ = self.poll.registry().deregister(stream);
        }
    }

    pub(super) fn fail_all(&mut self, message: String) {
        for (_, pending) in self.pending.drain() {
            pending.fail(message.clone());
        }
        self.listeners.clear();
        self.connections.clear();
    }
}
