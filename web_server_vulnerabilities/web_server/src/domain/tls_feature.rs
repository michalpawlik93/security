use mio::{net::TcpStream, Registry, Token};
use rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    ServerConfig, ServerConnection,
};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    sync::Arc,
};

pub struct Tls {
    pub tls_connection: TlsConnection,
}

/// Create key.pem and cert.pem in cert directory
/// You can use OpenSSL
impl Tls {
    pub fn new(socket: TcpStream, token: Token) -> io::Result<Self> {
        let certs = read_certs("cert/cert.pem")?;
        let keys = read_private_keys("cert/key.pem")?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, keys)
            .map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "invalid certificate/key pair")
            })?;
        let tls_conn = ServerConnection::new(Arc::new(config.clone())).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "invalid certificate/key pair")
        })?;
        Ok(Tls {
            tls_connection: TlsConnection::new(socket, token, tls_conn),
        })
    }
}

fn read_certs(file_path: &str) -> Result<Vec<CertificateDer<'static>>, io::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let certs: Result<Vec<_>, io::Error> = certs(&mut reader).collect();

    certs
}

fn read_private_keys(file_path: &str) -> Result<PrivateKeyDer<'static>, io::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let keys: Result<Vec<_>, io::Error> = pkcs8_private_keys(&mut reader).collect();

    keys.and_then(|mut keys| {
        keys.pop()
            .map(|key| PrivateKeyDer::Pkcs8(key))
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no valid private key found"))
    })
}

pub struct TlsConnection {
    pub socket: TcpStream,
    token: mio::Token,
    tls_conn: ServerConnection,
    closing: bool,
}

impl TlsConnection {
    fn new(socket: TcpStream, token: mio::Token, tls_conn: ServerConnection) -> Self {
        Self {
            socket,
            token,
            tls_conn,
            closing: false,
        }
    }

    pub fn handle_event(&mut self, registry: &Registry, event: &mio::event::Event) {
        if event.is_readable() {
            self.do_tls_read();
            self.try_plain_read();
        }

        if event.is_writable() {
            self.write_tls();
        }

        if self.closing {
            let _ = self.socket.shutdown(std::net::Shutdown::Both);
            self.deregister(registry);
        } else {
            self.reregister(registry);
        }
    }

    fn do_tls_read(&mut self) {
        match self.tls_conn.read_tls(&mut self.socket) {
            Err(err) => {
                if let io::ErrorKind::WouldBlock = err.kind() {
                    return;
                }
                eprintln!("read error {:?}", err);
                self.closing = true;
                return;
            }
            Ok(0) => {
                eprintln!("EOF - client closed connection.");
                self.closing = true;
                return;
            }
            Ok(_) => {}
        };

        if let Err(err) = self.tls_conn.process_new_packets() {
            eprintln!("cannot process packet: {:?}", err);
            self.do_tls_write_and_handle_error();

            self.closing = true;
        }
    }

    fn do_tls_write_and_handle_error(&mut self) {
        let rc = self.tls_write();
        if rc.is_err() {
            eprintln!("write failed {:?}", rc);
            self.closing = true;
        }
    }

    fn tls_write(&mut self) -> io::Result<usize> {
        self.tls_conn.write_tls(&mut self.socket)
    }

    fn try_plain_read(&mut self) {
        if let Ok(io_state) = self.tls_conn.process_new_packets() {
            if let Some(mut early_data) = self.tls_conn.early_data() {
                let mut buf = Vec::new();
                early_data.read_to_end(&mut buf).unwrap();

                if !buf.is_empty() {
                    eprintln!("early data read {:?}", buf.len());
                    self.incoming_plaintext(&buf);
                    return;
                }
            }

            if io_state.plaintext_bytes_to_read() > 0 {
                let mut buf = vec![0u8; io_state.plaintext_bytes_to_read()];

                self.tls_conn.reader().read_exact(&mut buf).unwrap();

                eprintln!("plaintext read {:?}", buf.len());
                self.incoming_plaintext(&buf);
            }
        }
    }

    fn incoming_plaintext(&mut self, buf: &[u8]) {
        self.send_http_response_once();
    }

    fn send_http_response_once(&mut self) {
        let response =
            b"HTTP/1.0 200 OK\r\nConnection: close\r\n\r\nHello world from tls server\r\n";
        self.tls_conn.writer().write_all(response).unwrap();
        self.tls_conn.send_close_notify();
    }

    fn write_tls(&mut self) {
        if self.tls_conn.wants_write() {
            let _ = self.tls_conn.write_tls(&mut self.socket);
        }
    }

    fn deregister(&mut self, registry: &Registry) {
        registry.deregister(&mut self.socket).unwrap();
    }

    fn reregister(&mut self, registry: &mio::Registry) {
        let event_set = self.event_set();
        registry
            .reregister(&mut self.socket, self.token, event_set)
            .unwrap();
    }

    fn event_set(&self) -> mio::Interest {
        let rd = self.tls_conn.wants_read();
        let wr = self.tls_conn.wants_write();

        if rd && wr {
            mio::Interest::READABLE | mio::Interest::WRITABLE
        } else if wr {
            mio::Interest::WRITABLE
        } else {
            mio::Interest::READABLE
        }
    }

    pub fn is_closed(&self) -> bool {
        self.closing
    }
}
