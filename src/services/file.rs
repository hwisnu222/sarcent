
use std::{error::Error,sync::Arc};

use futures::future::join_all;
use tokio::fs::File;
use tonic::{transport::Channel};
use tokio::io::AsyncReadExt;

use crate::{services::client::FileServiceClusterClient, repository::vnode::VnodeRepository, servers::master::file::{FileChunk, SearchRequest, SearchResponse, file_service_client::{self, FileServiceClient}}};

const CHUNK_SIZE: usize = 2 * 1024 * 1024;

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

pub async fn search_files(filename: String) -> Result<Vec<SearchResponse>, Box<dyn Error>>{
    let repo = VnodeRepository::new().await?;
    let cluster = FileServiceClusterClient::init().await?;
    let cluster_arc = Arc::new(cluster);

    let servers = repo.get_servers().await?;
    let mut tasks = Vec::new();
    
    // use tread async to get all server
    // to minimum usage of ram
    // because if create connection every hanshake will make more time to initial
    for server in servers{
        let file_name = filename.clone();
        let cluster_clone  = Arc::clone(&cluster_arc);

        let task = tokio::spawn(async move{
            if let Some(mut client) = cluster_clone.get_client(&server){
                let search = client.search_file(SearchRequest{file_name}).await?;
                let info = search.into_inner();

                Ok::<SearchResponse, Box<dyn Error + Send + Sync>>(info)
            }else{
                Err("failed connect to client".into())
            }
        });
        tasks.push(task);
    }


    let results = join_all(tasks).await;

    let responses: Vec<SearchResponse> = results.into_iter()
        .filter_map(|r|{
            match r {
                Ok(Ok(search_res)) =>{
                    Some(search_res)
                }
                Ok(Err(grpc_err))=>{
                    println!("failed search file. Error: {}", grpc_err);
                    None
                }
                Err(tokio_err)=>{
                    println!("process connect cancelled. Error: {}", tokio_err);
                    None
                }
            }
        }).collect();

    Ok(responses)
}
