use std::sync::{Arc, RwLock};
use awc::{Client as HttpClient, http::StatusCode};
use futures::TryFutureExt;
use logs::{logfile::block::Block, bbuff::absbuff::ByteBuff, logqueue::PreparedRecord};
use super::*;

pub struct QueueClient
{
    pub http_client: Arc<HttpClient>,
    pub base_url: String
}

impl QueueClient
{
    /// Получение версии
    pub async fn version(&self) -> Result<QueueApiVersion,ClientError> {
        let mut res = 
            self.http_client.get(format!("{}/queue/version", &self.base_url)).send().await?;
        if res.status() != StatusCode::OK {
            return Err(ClientError::Status { 
                code: res.status().as_u16(), 
                body: std::str::from_utf8( &res.body().await.unwrap() ).unwrap().to_string()
            });
        }
        Ok(res.json::<QueueApiVersion>().await?)
    }

    pub async fn files(&self) -> Result<LogFiles,ClientError> {
        let mut res = 
            self.http_client.get(format!("{}/queue/log/files", &self.base_url)).send().await?;
        if res.status() != StatusCode::OK {
            return Err(ClientError::Status { 
                code: res.status().as_u16(), 
                body: std::str::from_utf8( &res.body().await.unwrap() ).unwrap().to_string()
            });
        }
        Ok(res.json::<LogFiles>().await?)
    }

    pub async fn tail_id(&self) -> Result<TailId,ClientError> {
        let mut res = 
            self.http_client.get(format!("{}/queue/tail/id", &self.base_url)).send().await?;
        if res.status() != StatusCode::OK {
            return Err(ClientError::Status { 
                code: res.status().as_u16(), 
                body: std::str::from_utf8( &res.body().await.unwrap() ).unwrap().to_string()
            });
        }
        Ok(res.json::<TailId>().await?)
    }

    pub async fn switch_tail(&self) -> Result<TailSwitch,ClientError> {
        let mut res = 
            self.http_client.get(format!("{}/queue/tail/switch", &self.base_url)).send().await?;
        if res.status() != StatusCode::OK {
            return Err(ClientError::Status { 
                code: res.status().as_u16(), 
                body: std::str::from_utf8( &res.body().await.unwrap() ).unwrap().to_string()
            });
        }
        Ok(res.json::<TailSwitch>().await?)
    }

    pub async fn raw_record(&self, rid: RecId) -> Result<Vec<u8>, ClientError> {
        let mut res = 
            self.http_client.get(format!("{base}/queue/record/{lid}/{bid}/raw", 
                base = &self.base_url,
                lid = rid.log_id,
                bid = rid.block_id
            )).send().await?;

        if res.status() != StatusCode::OK {
            return Err(ClientError::Status { 
                code: res.status().as_u16(), 
                body: std::str::from_utf8( &res.body().await.unwrap() ).unwrap().to_string()
            });
        }

        let data = res.body().await?;
        let data = data.to_vec();

        Ok(data)
    }

    pub async fn record(&self, rid: RecId) -> Result<Block, ClientError> {
        self.raw_record(rid).and_then(|data| async move {
            let bbuff = ByteBuff {
                data: Arc::new(RwLock::new(data.clone())),
                resizeable: true,
                max_size: None
            };

            let block = 
                Block::read_from(
                    0u64, &bbuff
                ).map(|e|e.0)
                .map_err(|e| ClientError::ParseBlock(e))?;

            Ok(block)
        }).await
    }

    // pub async fn write_at<V:Into<Block>>(&self, rid:RecId, value:V) -> Result<(),ClientError> {
    //     let block: Block = value.into();

    //     self.http_client.post(format!("{base}/queue/record/{lid}/{bid}/raw",
    //         base=&self.base_url,
    //         lid=rid.log_id,
    //         bid=rid.block_id,
    //     ))
    //     .send_body(body);

    //     todo!()
    // }
}

#[test]
fn test_client() {
    use actix_rt::System;

    System::new().block_on(async {
        let client = QueueClient {
            http_client: Arc::new(HttpClient::default()),
            base_url: "http://localhost:8080".to_string()
        };

        let ver = client.version().await.unwrap();
        println!("{:?}", ver);

        println!("files");
        println!("{:?}", client.files().await.unwrap() );

        println!("read block");
        
        let block = client.record(RecId::new(1, 2) ).await.unwrap();
        println!("options");
        for (k,v) in block.head.block_options {
            println!("  {k}={v}");
        }
    })
}
