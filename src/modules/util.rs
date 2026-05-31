use tokio::fs::File;
use tonic::transport::Channel;
use tokio::io::AsyncReadExt;

use crate::modules::worker::masterworker::{self, FileChunk, StorageRequest, StorageResponse, file_service_client::FileServiceClient};

const CHUNK_SIZE: usize = 2 * 1024 * 1024;

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


pub async fn get_storage_info(ip_server: String) -> Result<StorageResponse, Box<dyn std::error::Error>>{
    println!("ip: {}", ip_server);
    let mut client = masterworker::master_worker_client::MasterWorkerClient::connect(ip_server).await?;
    let request = tonic::Request::new(StorageRequest{});
    let info =  client.get_storage_info(request).await?;
    Ok(info.into_inner())
}
