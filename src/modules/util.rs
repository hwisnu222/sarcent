use tokio::fs::File;
use tonic::transport::Channel;
use tokio::io::AsyncReadExt;

use crate::modules::worker::masterworker::{FileChunk, file_service_client::FileServiceClient};

const CHUNK_SIZE: usize = 2 * 1024 * 1024;

pub async fn upload_stream_file(client: &mut FileServiceClient<Channel>, file_path: &str) -> Result<String, Box<dyn std::error::Error>>{
    let mut file = File::open(file_path).await?;
    let filename = std::path::Path::new(file_path)
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
                    yield FileChunk{
                        filename: filename.clone(),
                        data: buffer[..n].to_vec(),
                        offset,
                        is_last: false,
                    };
                    offset += n as u64;
                }
                Err(_)=>{break}
            };

           
        };

         yield FileChunk{
                filename,
                data: vec![],
                offset,
                is_last: true,
        };
    };

    let res = client.upload_file(stream).await?;
    let r = res.into_inner();

    Ok(r.file_id)
}
