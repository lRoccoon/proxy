use proxy::*;
use tokio::net::TcpListener;
use tracing_log::log::debug;
#[tokio::main]
async fn main() {
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        debug!("accept from {addr:?}");
        let mut proxy = TCPProxy::new(stream);
        tokio::spawn(async move {
            let _ = proxy.run("nb.byted.org:80").await;
        });
    }
}
