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
use uuid::Uuid;
use walkdir::WalkDir;
use std::{env, error::Error, fs, path::{Path}};
use tonic::{ transport::{Channel}};
use crate::{repository::metadata::{MetadataItem, MetadataRepository}, servers::master::file::file_service_client::FileServiceClient};
use crate::servers::master::file::{FileChunk, FileInfo, SearchRequest, SearchResponse, UploadResponse};
use crate::{repository::vnode::VnodeRepository, services::{file::upload_stream_file, storage::get_storage_info}};



pub async fn run(source: String, tls: bool)->Result<(), Box<dyn Error>>{
    let repo = VnodeRepository::new().await?;
    let metadata_repository = MetadataRepository::new().await?;
    println!("master is running, source: {}", source);
    let servers = repo.get_servers().await?;

    let root_dir = Path::new(&source);
    if let Err(e) = env::set_current_dir(&root_dir){
        eprintln!("failed change workdir to {}. Error: {}",root_dir.display(), e);
        return Err(e.into());
    }

    // walkdir to get all file in current directory
    for entry in WalkDir::new("."){
        let entry_file = entry?;
        let path_file = entry_file.path();

        if path_file.is_file(){
            for url in &servers{
                let limit_storage_perc: f64  = 90.0 as f64;
                let protocol: &str = if tls {"https"} else {"http"};
                let host: String= format!("{}://{}", protocol, url);
                let storage_info = get_storage_info(host).await?;

                let file_size = entry_file.metadata()?.len();

                println!("available_space: {}", storage_info.available_space);
                // send file if storage_server < limit_staroage and
                // file_size < available_space_server
                if storage_info.usage_percent < limit_storage_perc && file_size < storage_info.available_space{
                    println!("file {} moving to {}", path_file.to_string_lossy(), url );

                    let protocol = if tls {"https"} else {"http"};
                    let server_url = format!("{}://{}",protocol, url);
                    let channel = Channel::from_shared(server_url)?.connect().await?;
                    let mut client = FileServiceClient::new(channel);

                    match upload_stream_file(&mut client, path_file.to_string_lossy().to_string()).await {
                        Ok(file_id)=>{
                            println!("success move file, {}", file_id);
                            let filename = path_file.file_name().map(|s| s.display().to_string()).unwrap_or_default();
                            let parent = path_file.parent().map(|s| s.display().to_string()).unwrap_or_default();

                            let metadata = MetadataItem{
                                name: filename,
                                path: parent,
                                size_bytes: file_size as u32,
                            };

                            if let Ok(_) =  metadata_repository.add_metadata(metadata).await{
                                println!("metadata is added");
                            }else{
                                println!("failed add metadata");
                            }
                        
                            fs::remove_file(path_file)?;
                            break;
                        }
                        Err(e)=>{
                            eprintln!("failed upload file. Error: {}", e);
                        }
                    }
                }else{
                    println!("server can't to saving, storage is fulled");
                    continue;
                }


            }
        }
    };

    Ok(())

}

#[derive(Clone)]
pub struct StorageData;

#[tonic::async_trait]
impl StorageService for StorageData{
    async fn get_storage_info(&self, _request: tonic::Request<StorageRequest>)-> Result<tonic::Response<StorageResponse>, tonic::Status>{
        let disks = Disks::new_with_refreshed_list();
        

        let disk = disks.first().ok_or_else(|| {tonic::Status::internal("no disk found")})?;
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;

        let response = StorageResponse{
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

    async fn search_file(&self, request: Request<SearchRequest>)-> Result<Response<SearchResponse>, Status>{
        let file_name_search = request.into_inner().file_name;
        println!("search file query: {}", file_name_search);

        // parse data option in cli argument
        // then get value option 'target' directory
        // walkdir inside that directory
        // filter with match request
        let mut files: Vec<FileInfo> = Vec::new();

        for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()){
            if entry.file_type().is_file(){
                if let Some(file_name_str) = entry.file_name().to_str(){
                    let is_match = file_name_str.to_lowercase().contains(&file_name_search.to_lowercase());

                    if is_match{
                        let path = Path::new(file_name_str);
                        let file_size = fs::metadata(path)?.len();

                        let item: FileInfo = FileInfo{
                            file_name: entry.path().display().to_string(),
                            file_size_bytes: file_size as i64
                        };
                        files.push(item);

                    }
                }
            }
        }

        let res = SearchResponse{
            files: files.clone(),
            total_found: files.len() as i32,
        };
        Ok(Response::new(res))
    }
}


