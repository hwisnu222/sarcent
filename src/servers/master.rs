use tracing::{debug, error, info};
use walkdir::WalkDir;
use std::{env, error::Error, fs, path::{Path}};
use tonic::{ transport::{Channel}};
use crate::{repository::metadata::{MetadataItem, MetadataRepository}, servers::worker::file::file_service_client::FileServiceClient, utils::host::get_host};
use crate::{repository::vnode::VnodeRepository, services::{file::upload_stream_file, storage::get_storage_info}};

pub async fn run(source: String)->Result<(), Box<dyn Error>>{
    let repo = VnodeRepository::new().await?;
    let metadata_repository = MetadataRepository::new().await?;
    
    info!("master is running...");
    info!("source: {}", source);

    let servers = repo.get_servers().await?;

    let root_dir = Path::new(&source);
    if let Err(e) = env::set_current_dir(&root_dir){
        error!("failed change workdir to {}. Error: {}",root_dir.display(), e);
        return Err(e.into());
    }

    // walkdir to get all file in current directory
    for entry in WalkDir::new("."){
        let entry_file = entry?;
        let path_file = entry_file.path();

        if path_file.is_file(){
            for url in &servers{
                let limit_storage_perc: f64  = 90.0 as f64;
                let host: String = get_host(url);
                let storage_info = get_storage_info(host.clone()).await?;

                let file_size = entry_file.metadata()?.len();

                debug!("available_space: {}", storage_info.available_space);
                // send file if storage_server < limit_staroage and
                // file_size < available_space_server
                if storage_info.usage_percent < limit_storage_perc && file_size < storage_info.available_space{
                    info!("file {} moving to {}", path_file.to_string_lossy(), url );

                    let channel = Channel::from_shared(host)?.connect().await?;
                    let mut client = FileServiceClient::new(channel);

                    match upload_stream_file(&mut client, path_file.to_string_lossy().to_string()).await {
                        Ok(file_id)=>{
                            info!("{} is moved", file_id);
                            let filename = path_file.file_name().map(|s| s.display().to_string()).unwrap_or_default();
                            let parent = path_file.parent().map(|s| s.display().to_string()).unwrap_or_default();

                            let metadata = MetadataItem{
                                name: filename,
                                path: parent,
                                size_bytes: file_size as u32,
                                server_address: url.to_string(),
                            };

                            match metadata_repository.add_metadata(metadata).await{
                                Ok(_)=>{
                                    info!("metadata has been added");
                                }
                                Err(e)=>{
                                    error!("{}", e);
                                }
                            }

                            fs::remove_file(path_file)?;
                            break;
                        }
                        Err(e)=>{
                            error!("failed upload file. Error: {}", e);
                        }
                    }
                }else{
                    error!("server can't save data, storage is full");
                    continue;
                }
            }
        }
    };

    Ok(())
}
