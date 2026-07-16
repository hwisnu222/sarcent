
use std::{env, error::Error, path::Path};
use tonic::{ transport::{Server}};
use tracing::{error, info};
use crate::servers::master::{FileUploadData, StorageData, file::file_service_server::FileServiceServer, storage::storage_service_server::StorageServiceServer};

pub async fn run(target: String ) -> Result<(), Box<dyn Error>>{
    info!("worker is running....");
    info!("target: {}", target);

    let address = "0.0.0.0:50051".parse()?;

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
