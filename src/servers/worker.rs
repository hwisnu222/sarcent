pub mod storage {
    tonic::include_proto!("storage");
}
pub mod file {
    tonic::include_proto!("file");
}

use storage::storage_service_server::{StorageService};
use file::file_service_server::{FileService};
use storage::{StorageRequest, StorageResponse};
use sysinfo::Disks;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tonic::{Request, Response, Status, Streaming};
use tracing::{error, info};
use uuid::Uuid;
use std::{env, error::Error,path::{Path}};
use tonic::{ transport::{Server}};

use crate::servers::worker::{file::{FileChunk, UploadResponse, file_service_server::FileServiceServer}, storage::storage_service_server::StorageServiceServer};

pub async fn run(target: String, port: u16) -> Result<(), Box<dyn Error>>{
    let address = format!("0.0.0.0:{}", port).parse()?;

    info!("worker is running at {}", address);
    info!("target: {}", target);

    let root_dir = Path::new(&target);
    if let Err(e) = env::set_current_dir(&root_dir){
        error!("failed change workdir to {}. Error: {}",root_dir.display(), e);
        return Err(e.into());
    }

    // create serve to receive request from master
    Server::builder()
        // add struct fileuploadservice to register on server
        .add_service(FileServiceServer::new(FileUploadData))
        .add_service(StorageServiceServer::new(StorageData))
        .serve(address)
        .await?;
    Ok(())
}


#[derive(Clone)]
pub struct StorageData;

#[tonic::async_trait]
impl StorageService for StorageData{
    async fn get_storage_info(&self, _request: tonic::Request<StorageRequest>)-> Result<tonic::Response<StorageResponse>, tonic::Status>{
        let disks = Disks::new_with_refreshed_list();
        
        let disk = disks.first().ok_or_else(|| {tonic::Status::internal("no disk found")})?;
        let total_space = disk.total_space();
        let available_space = disk.available_space();
        let used_space = total_space - available_space;
        let usage_percent = if total_space > 0 {(used_space as f64 / total_space as f64)*100.0} else {0.0};

        let response = StorageResponse{
            total_space,
            used_space,
            available_space,
            usage_percent
        };
        Ok(tonic::Response::new(response))
    }
}

pub struct FileUploadData;

// this trait for response from client
// if client upload file from the master worker, the worker server will catch request and return response
#[tonic::async_trait]
impl FileService for FileUploadData{
    async fn upload_file(&self, request: Request<Streaming<FileChunk>>)-> Result<Response<UploadResponse>, Status>{
        let mut stream = request.into_inner();
        let mut file: Option<File> = None;
        let mut total_size = 0u64;
        let mut filename = String::new();

        // catch stream from client and save to file
        while let Some(chunk) = stream.message().await?{
            if file.is_none(){
                filename = chunk.filename.clone();

                let path = format!("{}/{}",chunk.path, filename);
                std::fs::create_dir_all(chunk.path).unwrap();

                let raw_file = File::create(&path).await.map_err(|e| {
                    Status::internal(format!("{:?}", e))
                })?;
                file = Some(raw_file);
            }

            let f = file.as_mut().unwrap();
            f.write_all(&chunk.data).await
                .map(|e| Status::internal(format!("{:?}", e)))?;
            total_size += chunk.data.len() as u64;

            // flush chunk data in memory so the memory not fulled with trash chunk unused
            if chunk.is_last {
                f.flush().await.unwrap();
                break;
            }
        }

        let result = UploadResponse{
            file_id: Uuid::new_v4().to_string() ,
            message: format!("success upload file. filename: {}, filesize: {}",filename, total_size / 1024 / 1024),
            status: "success".to_string()
        };
        Ok(tonic::Response::new(result))
    }
}


