
use std::{error::Error};

use tokio::fs::File;
use tonic::{transport::Channel};
use tokio::io::AsyncReadExt;

use crate::{servers::worker::file::{FileChunk, file_service_client::{self, FileServiceClient}}};

const CHUNK_SIZE: usize = 2 * 1024 * 1024; // 2Mb

pub async fn client_file_service(ip_server: &str) -> Result<FileServiceClient<Channel>, Box<dyn Error>>{
    let host = format!("http://{}", ip_server);
    let client = file_service_client::FileServiceClient::connect(host).await?;
    Ok(client)
}

pub async fn upload_stream_file(client: &mut FileServiceClient<Channel>, file_path: String) -> Result<String, Box<dyn std::error::Error>>{
    let mut file = File::open(&file_path).await?;
    let path = std::path::PathBuf::from(file_path);
    let filename =path 
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let stream = async_stream::stream! {
        let mut offset = 0u64;
        let mut buffer = vec![0u8; CHUNK_SIZE];

        loop{
            match file.read(&mut buffer).await{
                Ok(0)=>break,
                Ok(n)=>{
                    if let Some(parent_path) = path.parent(){
                        yield FileChunk{
                            filename: filename.clone(),
                            data: buffer[..n].to_vec(),
                            offset,
                            is_last: false,
                            path: parent_path.to_string_lossy().into_owned(),
                        };
                        offset += n as u64;
                    };
                }
                Err(_)=>{break}
            };
        };

        if let Some(parent_path) = path.parent(){
            yield FileChunk{
                    filename,
                    data: vec![],
                    offset,
                    is_last: true,
                    path: parent_path.to_string_lossy().into_owned(),
            };
        }
    };

    let res = client.upload_file(stream).await?;
    let r = res.into_inner();

    Ok(r.file_id)
}

