use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::time::Duration;
mod domain;

const MAX_SOCKETS: usize = 32;
const SERVER_ADDRES: &str = "127.0.0.1:7878";
const SERVER_TOKEN: Token = Token(0);
fn main() {
    let mut poll = Poll::new().expect("Failed to create Poll instance");
    let mut server = TcpListener::bind(SERVER_ADDRES.parse().expect("Failed to parse address"))
        .expect("Failed to bind TcpListener");

    poll.registry()
        .register(&mut server, SERVER_TOKEN, Interest::READABLE)
        .expect("Failed to register listener");
    let mut events = Events::with_capacity(128);

    let mut next_socket_index = 1;
    let mut sockets = HashMap::new();

    println!("Server listening on {SERVER_ADDRES}");

    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))
            .expect("Polling failed");

        for event in events.iter() {
            match event.token() {
                SERVER_TOKEN => loop {
                    match server.accept() {
                        Ok((mut socket, client_address)) => {
                            if next_socket_index == MAX_SOCKETS {
                                panic!("Max socket reached. Shuting down the server");
                            }
                            println!("Creating socket for {client_address}");
                            let client_request_token = Token(next_socket_index);
                            next_socket_index += 1;

                            let _ = poll.registry().register(
                                &mut socket,
                                client_request_token,
                                Interest::READABLE,
                            );

                            sockets.insert(client_request_token, socket);
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            break;
                        }
                        e => panic!("err={:?}", e),
                    }
                },
                token if sockets.contains_key(&token) => {
                    if let Some(socket) = sockets.get_mut(&token) {
                        if event.is_readable() {
                            let _ = handle_client_request(socket);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

fn handle_client_request(socket: &mut TcpStream) -> io::Result<()> {
    println!("Connection with client established!");

    let mut http_request = [0; 1024];

    match socket.read(&mut http_request) {
        Ok(0) => {
            println!("EOF Client closed the connection.");
            return Ok(());
        }
        Ok(bytes_read) => {
            let request_str = String::from_utf8_lossy(&http_request[..bytes_read]);
            println!("Received request: {}", request_str.trim());

            if request_str.contains("\r\n\r\n") {
                let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!";

                socket.write_all(response.as_bytes())?;
            }
        }
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
            println!("WouldBlock error, will retry later");
            return Ok(());
        }
        Err(e) => {
            println!("Error while reading: {:?}", e);
            return Err(e);
        }
    }

    Ok(())
}
