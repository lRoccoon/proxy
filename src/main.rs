use clap::Parser;
use proxy::*;
use tokio::net::TcpListener;
use tracing_log::log::debug;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Config {
    #[arg(short, long)]
    upstream: String,
    #[arg(short, long)]
    bind: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let cfg = Config::parse();
    let listener = TcpListener::bind(cfg.bind).await.unwrap();
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        debug!("accept from {addr:?}");
        let mut proxy = TCPProxy::new(cfg.upstream.clone());
        tokio::spawn(async move {
            let _ = proxy.run(stream).await;
        });
    }
}
