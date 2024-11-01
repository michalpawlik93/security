use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::time::Duration;

use crate::domain::global_context;
use crate::domain::tls_feature;

pub struct TcpServer {
    poll: Poll,
    server: TcpListener,
    sockets: HashMap<i32, SocketMode>,
    next_socket_id: i32,
}

enum SocketMode {
    Http(TcpStream),
    Https(tls_feature::TlsConnection),
}

impl TcpServer {
    const MAX_SOCKETS: usize = 32;
    const SERVER_ADDRESS: &'static str = "127.0.0.1:7878";
    const SERVER_TOKEN: Token = Token(0);

    pub fn new() -> io::Result<Self> {
        let poll: Poll = Poll::new()?;
        let mut server = TcpListener::bind(
            Self::SERVER_ADDRESS
                .parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
        )?;
        poll.registry().register(
            &mut server,
            Self::SERVER_TOKEN,
            Interest::READABLE | Interest::WRITABLE,
        )?;
        Ok(TcpServer {
            poll,
            server,
            sockets: HashMap::new(),
            next_socket_id: 1,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut events = Events::with_capacity(128);
        println!("Server listening on {}", Self::SERVER_ADDRESS);

        loop {
            self.poll
                .poll(&mut events, Some(Duration::from_millis(100)))?;
            for event in events.iter() {
                match event.token() {
                    Self::SERVER_TOKEN => self.accept_connections()?,
                    token => {
                        if let Some(socket_id) = self.get_socket_id_from_token(token) {
                            self.process_client_request(socket_id, event)?;
                        }
                    }
                }
            }
        }
    }

    fn accept_connections(&mut self) -> io::Result<()> {
        while let Ok((socket, client_address)) = self.server.accept() {
            if self.sockets.len() >= Self::MAX_SOCKETS {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Max sockets reached: {}", client_address),
                ));
            }
            println!("Accepted connection from {}", client_address);

            let socket_id = self.next_socket_id;
            self.next_socket_id += 1;
            let token = Token(socket_id as usize);

            let mut socket_state = if cfg!(feature = "tls") {
                SocketMode::Https(tls_feature::Tls::new(socket, token)?.tls_connection)
            } else {
                SocketMode::Http(socket)
            };

            self.poll.registry().register(
                match &mut socket_state {
                    SocketMode::Http(s) => s,
                    SocketMode::Https(t) => &mut t.socket,
                },
                token,
                Interest::READABLE | Interest::WRITABLE,
            )?;

            self.sockets.insert(socket_id, socket_state);
        }
        Ok(())
    }

    fn process_client_request(
        &mut self,
        socket_id: i32,
        event: &mio::event::Event,
    ) -> io::Result<()> {
        if let Some(socket_state) = self.sockets.get_mut(&socket_id) {
            match socket_state {
                SocketMode::Http(socket) => {
                    handle_direct_socket(socket, socket_id)?;
                }
                SocketMode::Https(tls_connection) => {
                    tls_connection.handle_event(&self.poll.registry(), event);
                    if tls_connection.is_closed() {
                        self.sockets.remove(&socket_id);
                    }
                }
            }
        }
        Ok(())
    }

    fn get_socket_id_from_token(&self, token: Token) -> Option<i32> {
        if token.0 >= 1 && token.0 < i32::MAX as usize {
            Some(token.0 as i32)
        } else {
            None
        }
    }
}

fn handle_direct_socket(socket: &mut TcpStream, socket_id: i32) -> io::Result<()> {
    let mut http_request = [0; 1024];
    match socket.read(&mut http_request) {
        Ok(0) => {
            println!("EOF: Client closed the connection.");
            return Ok(());
        }
        Ok(bytes_read) => {
            let request_str = String::from_utf8_lossy(&http_request[..bytes_read]);
            println!("Received request: {}", request_str.trim());

            let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!";
            socket.write_all(response.as_bytes())?;
        }
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
            return Ok(());
        }
        Err(e) => {
            println!("{e}");
            global_context::GlobalContext::add_global_error(socket_id, e.to_string());
            return Ok(());
        }
    }
    Ok(())
}
