#![deny(missing_docs)]
//! The simplest echo server

use std::io;
use std::io::Read;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// A TCP server that echoes any message received from a client to all clients.
pub struct EchoServer {
    // The address on which the server is listening.
    address: SocketAddr,
    // The flag that indicates if the server is currently running.
    running: Arc<AtomicBool>,
}

impl EchoServer {
    const DELAY_MS: u64 = 100;

    /// Constructs a new EchoServer with the given address.
    pub fn new(address: SocketAddr) -> Self {
        EchoServer {
            address,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Sets the address on which the server is listening. The change will have no effect until the next call to start.
    pub fn set_address(&mut self, address: SocketAddr) {
        self.address = address
    }

    /// Gets the flag that indicates if the server is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Asynchronously starts the process of binding to the current address and accepting client connections.
    /// If the server is already bound or attempting to bind, this method has no effect.
    pub fn start(&mut self) {
        if self.running.swap(true, Ordering::Relaxed) {
            return;
        }
        let accept_address = self.address;
        let accept_running = Arc::clone(&self.running);
        thread::Builder::new()
            .name(String::from("JdnEcho-TcpListener-accept"))
            .spawn(move || {
                let _ = Self::accept_process(accept_address, accept_running);
            })
            .expect("failed to spawn thread");
    }

    /// Asynchronously stops the server process.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed)
    }

    fn accept_process(address: SocketAddr, accept_running: Arc<AtomicBool>) -> std::io::Result<()> {
        while accept_running.load(Ordering::Relaxed) {
            match TcpListener::bind(address) {
                Ok(listener) => {
                    listener.set_nonblocking(true)?;
                    match listener.accept() {
                        Ok((mut socket, addr)) => {
                            println!("Accepted connection from {}", addr);
                            if socket.set_nonblocking(true).is_err() {
                                continue;
                            }
                            thread::Builder::new()
                                .name(format!("JdnEcho-TcpListener-{}-read", addr))
                                .spawn(move || {
                                    let mut buf = Vec::new();
                                    match socket.read_to_end(&mut buf) {
                                        Ok(_) => {
                                            if !buf.is_empty() {
                                                match String::from_utf8(buf.drain(..).collect()) {
                                                    Ok(data) => {
                                                        println!("{}", data);
                                                    }
                                                    Err(e) => {
                                                        println!("Could not parse data: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            if e.kind() == io::ErrorKind::WouldBlock {
                                                thread::sleep(Duration::from_millis(
                                                    Self::DELAY_MS,
                                                ));
                                            } else {
                                                // return Err(e);
                                            }
                                        }
                                    }
                                })
                                .expect("failed to spawn thread");
                        }
                        Err(e) => {
                            if e.kind() == io::ErrorKind::WouldBlock {
                                thread::sleep(Duration::from_millis(Self::DELAY_MS));
                            } else {
                                return Err(e);
                            }
                        }
                    }
                }
                Err(_) => thread::sleep(Duration::from_millis(Self::DELAY_MS)),
            }
        }
        Ok(())
    }
}
