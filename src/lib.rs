use std::sync::Arc;

use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::Mutex,
};
use tracing_log::log::{debug, error, info};

pub struct TCPProxy {
    income_rx: Arc<Mutex<OwnedReadHalf>>,
    income_tx: Arc<Mutex<OwnedWriteHalf>>,
}

impl TCPProxy {
    pub fn new(income: TcpStream) -> Self {
        let (income_rx, income_tx) = income.into_split();
        Self {
            income_rx: Arc::new(Mutex::new(income_rx)),
            income_tx: Arc::new(Mutex::new(income_tx)),
        }
    }
    pub async fn run(&mut self, upstream_addr: &str) -> Result<(), io::Error> {
        let upstream = TcpStream::connect(upstream_addr).await?;
        let (mut upstream_rx, mut upstream_tx) = upstream.into_split();
        let mut income_rx = self.income_rx.clone().lock_owned().await;
        let mut income_tx = self.income_tx.clone().lock_owned().await;
        let mut upstream_buf = [0u8; 512];
        let mut income_buf = [0u8; 512];
        loop {
            let read_from_upstream = upstream_rx.read(&mut upstream_buf);
            let read_from_income = income_rx.read(&mut income_buf);
            tokio::select! {
                read_rst = read_from_upstream =>match read_rst{
                    Ok(read_n)=>{
                        debug!("read {read_n} bytes from upstream");
                        if read_n == 0{
                            info!("read from upstream end.");
                            return Ok(())
                        }
                        match income_tx.write(&upstream_buf[..read_n]).await{
                            Ok(write_n)=> debug!("write {write_n} bytes to income"),
                            Err(err)=>error!("write to income error: {err:?}"),
                        };
                    }
                    Err(err)=>{
                        error!("read from upstream error: {err:?}");
                        return Err(err)
                    },
                },
                read_rst = read_from_income=>match read_rst{
                    Ok(read_n)=>{
                        debug!("read {read_n} bytes from income");
                        if read_n == 0{
                            info!("read from income end.");
                            return Ok(())
                        }
                        match upstream_tx.write(&upstream_buf[..read_n]).await{
                            Ok(write_n)=> debug!("write {write_n} bytes to upstream"),
                            Err(err)=>error!("write to upstream error: {err:?}"),
                        };
                    }
                    Err(err)=>{
                        error!("read from income error: {err:?}");
                        return Err(err)
                    },
                },
            }
        }
    }
}
