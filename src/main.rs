use serde_derive::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

use clap::Parser;
use proxy::*;
use tokio::net::TcpListener;
use tracing_log::log::{debug, error, info};

const PKG_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE", help = "Config file path")]
    config: Option<PathBuf>,
    #[arg(short, long, value_name = "ADDRESS", help = "Upstream address")]
    upstream: Option<String>,
    #[arg(short, long, value_name = "ADDRESS", help = "Bind local address")]
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
        panic!("bind address and upstream not found, please set config file");
    }
    if args.bind.is_none() && args.upstream.is_none() {
        run_with_config(args.config).await;
    } else if args.bind.is_none() || args.upstream.is_none() {
        panic!("bind address and upstream not found, please set config file");
    } else {
        run("proxy", &args.bind.unwrap(), &args.upstream.unwrap()).await;
    }

    match tokio::signal::ctrl_c().await {
        Ok(()) => info!("stop proxy..."),
        Err(err) => error!("unable to listen for shutdown signal: {err:?}"),
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}

async fn run(name: &str, bind: &str, upstream: &str) {
    let listener = TcpListener::bind(bind).await.unwrap();
    info!("listen {name}({upstream}) on {bind}...");
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        info!("accept {name} connection from {addr:?}");
        let mut proxy = TCPProxy::new(name.to_string(), upstream.to_string());
        tokio::spawn(async move {
            let _ = proxy.run(stream).await;
        });
    }
}

async fn run_with_config(path: Option<PathBuf>) {
    let cfg: Config = match path {
        Some(path) => confy::load_path(path).unwrap(),
        None => confy::load(PKG_NAME, "config.toml").unwrap(),
    };
    for proxy in cfg.proxies {
        tokio::spawn(async move {
            run(&proxy.name, &proxy.bind, &proxy.upstream).await;
        });
    }
}
