use crate::servers::master::storage::{StorageRequest, StorageResponse, storage_service_client::StorageServiceClient};


pub async fn get_storage_info(ip_server: String) -> Result<StorageResponse, Box<dyn std::error::Error>>{
    println!("ip: {}", ip_server);
    let mut client = StorageServiceClient::connect(ip_server).await?;
    let request = tonic::Request::new(StorageRequest{});
    let info =  client.get_storage_info(request).await?;
    Ok(info.into_inner())
}

