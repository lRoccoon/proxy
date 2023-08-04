use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

use clap::Parser;
use proxy::*;
use tokio::{
    net::TcpListener,
    task::{JoinHandle, JoinSet},
};
use tracing_log::log::{debug, info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    #[arg(short, long)]
    upstream: Option<String>,
    #[arg(short, long)]
    bind: Option<String>,
}
#[derive(Default, Serialize, Deserialize)]
struct Config {
    proxies: Vec<Proxy>,
}
#[derive(Serialize, Deserialize)]
struct Proxy {
    name: String,
    bind: String,
    upstream: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    if args.config.is_none() && (args.bind.is_none() && args.upstream.is_none()) {
        panic!("config or upstream not found");
    }
    match args.config {
        Some(path) => {
            run_with_config(path).await;
        }
        None => {
            if args.bind.is_none() || args.upstream.is_none() {
                panic!("bind address or upstream not found");
            }
            run("proxy", &args.bind.unwrap(), &args.upstream.unwrap()).await;
        }
    }
}

async fn run(name: &str, bind: &str, upstream: &str) {
    let listener = TcpListener::bind(bind).await.unwrap();
    info!("listen {name}({upstream}) on {bind}...");
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        debug!("accept {name} connection from {addr:?}");
        let mut proxy = TCPProxy::new(name.to_string(), upstream.to_string());
        tokio::spawn(async move {
            let _ = proxy.run(stream).await;
        });
    }
}

async fn run_with_config(path: PathBuf) {
    let cfg: Config = confy::load_path(path).unwrap();
    let mut set = JoinSet::new();
    for proxy in cfg.proxies {
        set.spawn(async move {
            run(&proxy.name, &proxy.bind, &proxy.upstream).await;
        });
    }
    while set.join_next().await.is_some() {}
}
