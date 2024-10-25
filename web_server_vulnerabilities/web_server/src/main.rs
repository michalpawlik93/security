mod domain;

fn main() {
    let mut server = domain::tcp_server::TcpServer::new().expect("Failed to start the server");

    server.run();
}
