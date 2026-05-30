pub mod masterworker {
    tonic::include_proto!("masterworker");
}

use masterworker::master_worker_server::{MasterWorker};
use masterworker::file_service_server::{FileService};
use masterworker::{StorageRequest, StorageResponse};
use sysinfo::Disks;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tonic::{Request, Response, Status, Streaming};
use uuid::Uuid;

use crate::modules::worker::masterworker::{FileChunk, UploadResponse};

#[derive(Clone)]
pub struct WorkerService{
    pub worker_id: String
}

#[tonic::async_trait]
impl MasterWorker for WorkerService{
    async fn get_storage_info(&self, _request: tonic::Request<StorageRequest>)-> Result<tonic::Response<StorageResponse>, tonic::Status>{
        let disks = Disks::new_with_refreshed_list();
        

        let disk = disks.first().ok_or_else(|| {tonic::Status::internal("no disk found")})?;
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;

        let response = StorageResponse{
            worker_id: self.worker_id.clone(),
            total_space: total,
            used_space: used,
            available_space: available,
            usage_percent: if total > 0 {(used as f64 / total as f64)*100.0} else {0.0},
        };
        Ok(tonic::Response::new(response))
    }

    // fn monitor(&self, _request: tonic::Request<MasterCommand>)-> Result<tonic::Response<tonic::Streaming<WorkerStatus>>, tonic::Status>{
    //     let response = 
    //     Ok(tonic::Response::new())
    // }
    //
    
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


