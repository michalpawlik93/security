use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

const MITM_LISTENER: Token = Token(0);
const SERVER_ADDRESS: &str = "127.0.0.1:7878";
const MITM_ADDRESS: &str = "127.0.0.1:8081";

struct TcpConnectionPair {
    source: Arc<Mutex<TcpStream>>,
    target: Arc<Mutex<TcpStream>>,
}

fn main() -> std::io::Result<()> {
    let addr = MITM_ADDRESS.parse::<SocketAddr>().unwrap();
    let mut listener = TcpListener::bind(addr)?;
    let mut poll = Poll::new()?;
    poll.registry()
        .register(&mut listener, MITM_LISTENER, Interest::READABLE)?;

    let mut events = Events::with_capacity(128);
    let mut clients: HashMap<Token, Arc<Mutex<TcpConnectionPair>>> = HashMap::new();
    let mut servers: HashMap<Token, Arc<Mutex<TcpConnectionPair>>> = HashMap::new();
    let mut unique_token = 1;

    println!("MITM listening on: {}", MITM_ADDRESS);

    loop {
        poll.poll(&mut events, None)?;

        for event in &events {
            match event.token() {
                MITM_LISTENER => {
                    mitm_token_handle(
                        &mut listener,
                        &mut poll,
                        &mut unique_token,
                        &mut clients,
                        &mut servers,
                    )?;
                }
                token if clients.contains_key(&token) => {
                    token_handle(event.token(), &mut clients)?;
                }
                token if servers.contains_key(&token) => {
                    token_handle(event.token(), &mut servers)?;
                }
                _ => (),
            }
        }
    }
}

fn mitm_token_handle(
    listener: &mut TcpListener,
    poll: &mut Poll,
    unique_token: &mut usize,
    clients: &mut HashMap<Token, Arc<Mutex<TcpConnectionPair>>>,
    servers: &mut HashMap<Token, Arc<Mutex<TcpConnectionPair>>>,
) -> io::Result<()> {
    if let Ok((mut client, client_addr)) = listener.accept() {
        println!("Accepted client connection from {}", client_addr);

        let server_addr = SERVER_ADDRESS.parse::<SocketAddr>().unwrap();
        let mut server = TcpStream::connect(server_addr)?;

        let client_token = Token(*unique_token);
        *unique_token += 1;
        let server_token = Token(*unique_token);
        *unique_token += 1;

        poll.registry().register(
            &mut client,
            client_token,
            Interest::READABLE | Interest::WRITABLE,
        )?;
        poll.registry().register(
            &mut server,
            server_token,
            Interest::READABLE | Interest::WRITABLE,
        )?;

        let client_mutex = Arc::new(Mutex::new(client));
        let server_mutex = Arc::new(Mutex::new(server));

        let tcp_connection_pair1 = Arc::new(Mutex::new(TcpConnectionPair {
            source: client_mutex.clone(),
            target: server_mutex.clone(),
        }));

        let tcp_connection_pair2 = Arc::new(Mutex::new(TcpConnectionPair {
            source: server_mutex,
            target: client_mutex,
        }));

        clients.insert(client_token, tcp_connection_pair1);
        servers.insert(server_token, tcp_connection_pair2);

        println!("Client and server connections registered with tokens.");
    }
    Ok(())
}

fn token_handle(
    token: Token,
    tcp_connection_pair: &mut HashMap<Token, Arc<Mutex<TcpConnectionPair>>>,
) -> io::Result<()> {
    if let Some(connection) = tcp_connection_pair.get(&token) {
        let mut buffer = [0; 1024];

        let connection_pair = connection.lock().unwrap();

        let mut source_stream = connection_pair.source.lock().unwrap();

        match source_stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("Received from source: {:?}", request_str.trim());

                let mut target_stream = connection_pair.target.lock().unwrap();
                target_stream.write_all(&buffer[..bytes_read])?;
                println!("Forwarded data to target");
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                println!("Source read would block");
            }
            Err(e) => return Err(e),
            _ => (),
        }
    }
    Ok(())
}
