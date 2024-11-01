mod domain;

fn main() {
    let server = domain::tcp_server::TcpServer::new();
    match server {
        Ok(mut s) => {
            if let Err(e) = s.run() {
                eprintln!("Error occurred while running the server: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Error occurred while starting the server: {}", e);
            std::process::exit(1);
        }
    }
}
