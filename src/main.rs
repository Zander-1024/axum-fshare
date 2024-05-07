mod api;
mod cli;
mod zip_utils;

use clap::Parser;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();
    let ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
    let socket_addr = SocketAddr::new(ip, args.port);
    match api::build_app(&args) {
        Ok(app) => {
            let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
        Err(err) => println!("build app err:{}", err),
    }
}
