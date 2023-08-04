use tokio::{
    io::{self},
    net::TcpStream,
};
use tracing_log::log::{error, info};

pub struct TCPProxy {
    upstream_addr: String,
}

impl TCPProxy {
    pub fn new(upstream_addr: String) -> Self {
        Self { upstream_addr }
    }
    pub async fn run(&mut self, income: TcpStream) -> Result<(), io::Error> {
        let (mut income_rx, mut income_tx) = income.into_split();
        let upstream = TcpStream::connect(&self.upstream_addr).await?;
        let (mut upstream_rx, mut upstream_tx) = upstream.into_split();
        tokio::spawn(async move {
            let up = io::copy(&mut income_rx, &mut upstream_tx).await;
            match up {
                Ok(sent_n) => info!("send {sent_n} bytes to upstream"),
                Err(err) => error!("send to upstream error: {err:?}"),
            }
        });

        tokio::spawn(async move {
            let down = io::copy(&mut upstream_rx, &mut income_tx).await;
            match down {
                Ok(sent_n) => info!("send {sent_n} bytes to income"),
                Err(err) => error!("send to income error: {err:?}"),
            }
        });
        Ok(())
    }
}
