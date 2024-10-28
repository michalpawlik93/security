use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::time::Duration;

use crate::domain::global_context;

pub struct TcpServer {
    poll: Poll,
    server: TcpListener,
    sockets: HashMap<i32, TcpStream>,
    next_socket_id: i32,
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
                .expect("Could not start server"),
        )?;
        poll.registry()
            .register(&mut server, Self::SERVER_TOKEN, Interest::READABLE)?;

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
        while let Ok((mut socket, client_address)) = self.server.accept() {
            if self.sockets.len() == Self::MAX_SOCKETS {
                eprintln!(
                    "Max socket limit reached, rejecting connection from {}",
                    client_address
                );
                return Err(io::Error::new(io::ErrorKind::Other, "Max sockets reached"));
            }
            println!("Accepted connection from {}", client_address);

            let socket_id = self.next_socket_id;
            self.next_socket_id += 1;
            self.poll.registry().register(
                &mut socket,
                Token(socket_id as usize),
                Interest::READABLE,
            )?;

            self.sockets.insert(socket_id, socket);
        }
        Ok(())
    }

    fn process_client_request(
        &mut self,
        socket_id: i32,
        event: &mio::event::Event,
    ) -> io::Result<()> {
        if let Some(socket) = self.sockets.get_mut(&socket_id) {
            if event.is_readable() {
                handle_client_request(socket, self.next_socket_id)?;
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

fn handle_client_request(socket: &mut TcpStream, socket_id: i32) -> io::Result<()> {
    let mut http_request = [0; 1024];
    match socket.read(&mut http_request) {
        Ok(0) => {
            println!("EOF: Client closed the connection.");
            return Ok(());
        }
        Ok(bytes_read) => {
            let request_str = String::from_utf8_lossy(&http_request[..bytes_read]);
            println!("Received request: {}", request_str.trim());

            if request_str.contains("\r\n\r\n") {
                let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!";
                socket.write_all(response.as_bytes())?;
                socket.flush()?;
                socket.shutdown(std::net::Shutdown::Both)?;
            }
        }
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
            return Ok(());
        }
        Err(e) => {
            println!("Error while reading: {:?}", e);
            global_context::GlobalContext::add_global_error(socket_id, e.to_string());
            return Err(e);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::global_context::GlobalContext;
    use mio::net::TcpListener;
    use mio::{Events, Interest, Poll, Token};
    use std::net::{SocketAddr, TcpStream as StdTcpStream};
    use std::thread;
    use std::time::Duration;

    fn setup_listener() -> (TcpListener, SocketAddr) {
        let addr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(addr).unwrap();
        let local_addr = listener.local_addr().unwrap();
        (listener, local_addr)
    }

    #[test]
    fn test_handle_client_request_valid_request() {
        let (mut listener, addr) = setup_listener();
        let socket_id = 1;

        thread::spawn(move || {
            let mut client = StdTcpStream::connect(addr).unwrap();
            client.write_all(b"GET / HTTP/1.1\r\n\r\n").unwrap();
        });

        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(128);
        poll.registry()
            .register(&mut listener, Token(0), Interest::READABLE)
            .unwrap();

        poll.poll(&mut events, Some(Duration::from_millis(100)))
            .unwrap();
        for event in events.iter() {
            if event.token() == Token(0) && event.is_readable() {
                let (mut socket, _) = listener.accept().unwrap();

                let result = handle_client_request(&mut socket, socket_id);
                assert!(result.is_ok());
                assert!(GlobalContext::get_global_errors(socket_id).is_none());
            }
        }
    }
}
