#![deny(missing_docs)]
//! The simplest echo client

use std::io;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::net::TcpStream;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// A TCP client that can send and receive text to and from an echo server.
pub struct EchoClient {
    // The address to which the client should connect.
    address: SocketAddr,
    // The flag that indicates if the client should be attempting to connect.
    running: Arc<AtomicBool>,
    // The flag that indicates if the client is successfully connected to a server.
    connected: Arc<AtomicBool>,
    // The Sender used to send messages to the server.
    sender: Arc<Mutex<Option<mpsc::Sender<String>>>>,
}

impl EchoClient {
    const DELAY_MS: u64 = 100;

    /// Constructs a new EchoClient with the given address.
    pub fn new(address: SocketAddr) -> Self {
        EchoClient {
            address,
            running: Arc::new(AtomicBool::new(false)),
            connected: Arc::new(AtomicBool::new(false)),
            sender: Arc::new(Mutex::new(None)),
        }
    }

    /// Sets the address to which the client should connect. The change will have no effect until the next call to start.
    pub fn set_address(&mut self, address: SocketAddr) {
        self.address = address
    }

    /// Gets the flag that indicates if the client should be attempting to connect.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Gets the flag that indicates if the client is successfully connected to a server.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Sends the given message to the server. If the client is not currently connected, this method has no effect.
    pub fn send_message(&self, message: &str) {
        if let Some(msg_sender) = self.sender.lock().unwrap().deref() {
            let _ = msg_sender.send(message.to_owned());
        }
    }

    /// Asynchronously starts the process of connecting to the server and listening for data.
    /// If the client is already connected or attempting to connect, this method has no effect.
    pub fn start(&mut self) {
        if self.running.swap(true, Ordering::Relaxed) {
            return;
        }
        let connect_running = Arc::clone(&self.running);
        let connect_connected = Arc::clone(&self.connected);
        let connect_address = self.address.clone();
        let sender = Arc::clone(&self.sender);
        thread::Builder::new()
            .name(String::from("JdnEcho-TcpStream-connect"))
            .spawn(move || {
                while connect_running.load(Ordering::Relaxed) {
                    let stream_result = TcpStream::connect_timeout(
                        &connect_address,
                        Duration::from_millis(Self::DELAY_MS),
                    );
                    match stream_result {
                        Ok(stream) => {
                            println!("Successfully connected to {}", connect_address);
                            if let Err(_) = stream.set_nonblocking(true) {
                                continue;
                            }
                            connect_connected.store(true, Ordering::Relaxed);
                            let read_running = Arc::clone(&connect_running);
                            let read_connected = Arc::clone(&connect_connected);
                            let read_stream = stream.try_clone().unwrap();
                            let read_thread = thread::Builder::new()
                                .name(String::from("JdnEcho-TcpStream-read"))
                                .spawn(move || {
                                    let _ = EchoClient::read_process(
                                        read_stream,
                                        read_running,
                                        read_connected,
                                    );
                                })
                                .expect("failed to spawn thread");

                            let write_running = Arc::clone(&connect_running);
                            let write_connected = Arc::clone(&connect_connected);
                            let (write_sender, write_receiver) = mpsc::channel::<String>();
                            let write_thread = thread::Builder::new()
                                .name(String::from("JdnEcho-TcpStream-write"))
                                .spawn(move || {
                                    let _ = EchoClient::write_process(
                                        stream,
                                        write_running,
                                        write_connected,
                                        write_receiver,
                                    );
                                })
                                .expect("failed to spawn thread");
                            *sender.lock().unwrap() = Some(write_sender);

                            let _ = read_thread.join();
                            let _ = write_thread.join();
                        }
                        Err(_) => {
                            thread::sleep(Duration::from_millis(Self::DELAY_MS));
                        }
                    }
                }
            })
            .expect("failed to spawn thread");
    }

    /// Asynchronously disconnects from the server if a connection was established, and stops connection attempts.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed)
    }

    fn read_process(
        mut stream: TcpStream,
        read_running: Arc<AtomicBool>,
        read_connected: Arc<AtomicBool>,
    ) -> std::io::Result<()> {
        let mut buf = Vec::new();
        while read_running.load(Ordering::Relaxed) {
            match stream.read_to_end(&mut buf) {
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
                        thread::sleep(Duration::from_millis(Self::DELAY_MS));
                    } else {
                        read_connected.store(false, Ordering::Relaxed);
                        return Err(e);
                    }
                }
            }
            thread::sleep(Duration::from_millis(Self::DELAY_MS));
        }
        Ok(())
    }

    fn write_process(
        mut stream: TcpStream,
        write_running: Arc<AtomicBool>,
        write_connected: Arc<AtomicBool>,
        write_receiver: mpsc::Receiver<String>,
    ) -> std::io::Result<()> {
        while write_running.load(Ordering::Relaxed) {
            if let Ok(msg) = write_receiver.recv_timeout(Duration::from_millis(Self::DELAY_MS)) {
                if let Err(e) = stream.write_all(msg.as_bytes()) {
                    write_connected.store(false, Ordering::Relaxed);
                    return Err(e);
                }
            }
            if !write_connected.load(Ordering::Relaxed) {
                break;
            }
        }
        Ok(())
    }
}
