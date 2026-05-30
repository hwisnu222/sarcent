use std::{error::Error, net::SocketAddr, thread, time::Duration};
use tonic::{server, transport::{Channel, Server}};
use crate::modules::{hash::hash_to_coordinate, repository::Repository, util::upload_stream_file, worker::{FileUploadData, WorkerService, masterworker::{self, StorageRequest, file_service_client::FileServiceClient, file_service_server::FileServiceServer, master_worker_server::MasterWorkerServer}}};
use walkdir::WalkDir;

pub async fn master_runner(repo: Repository, interval: u64, source: String)->Result<(), Box<dyn Error>>{

    println!("master is running, target: {}", source);
    // list all worker
    let servers = repo.get_servers().await?;
    let node = repo.get_nodes().await?;

    loop {

        // TODO: walkdir to get all file in current directory
        for entry in WalkDir::new(&source){
            let entry_file = entry?;
            let path_file = entry_file.path();

            if let Some(file_name) = path_file.file_name(){
                // TODO: hashing every file to murmur3 with 64
                let hashed = hash_to_coordinate(&file_name.to_string_lossy());

                // TODO: get server ip based on hash >= hash_key server_address
                let get_server = node.ring.iter().find(|&&x| hashed > x);

                match get_server {
                    Some(value)=> {
                        let vnodes = node.vnode_map.clone();

                        if let Some(server_url) =  vnodes.get(&value){
                            // TODO: then use the result ip to send file
                            println!("file {} moving to {}", path_file.to_string_lossy(), server_url );
                            // let channel = Channel::from_shared(server_url.clone())?.connect().await?;
                            // let mut client = FileServiceClient::new(channel);
                            //
                            // match upload_stream_file(&mut client, &path_file.to_string_lossy()).await {
                            //     Ok(file_id)=>{
                            //         println!("success uppload, {}", file_id);
                            //
                            //         // TODO: remove file in folder master
                            //     }
                            //     Err(e)=>{
                            //         eprintln!("failed upload file. Error: {}", e);
                            //     }
                            // }
                        }

                    }
                    None=>{

                        // TODO: handle if hash file get last hash vnode then the file will move in
                        // hash vnode first ring, but if vnode full storage up to ring above until
                        // get vnode with empty storage
                        println!("server not found");
                    }
                }



            }


        };


        


        for worker_addr in servers.clone(){
            println!("Connecting to {}", worker_addr);
            let ip_server = format!("http://{}", worker_addr);

            match masterworker::master_worker_client::MasterWorkerClient::connect(ip_server).await{
                Ok(mut client)=> {
                    println!("connected");

                    let request = tonic::Request::new(StorageRequest{});

                    match client.get_storage_info(request).await{
                        Ok(response)=>{
                            let info = response.into_inner();
                            println!("usage: {:.2}%", info.usage_percent);

                        },
                        Err(_)=>{
                            println!("failed get info storage");
                        }
                    };
                },
                Err(_)=>{
                    println!("error connected");
                }
            }
        }
        thread::sleep(Duration::from_secs(interval));
    }
}


pub async fn worker_runner(master_addr: String, target: String ) -> Result<(), Box<dyn Error>>{
    println!("worker is running.... target: {}", target);
    let worker_service =  WorkerService{
        worker_id: format!("worker-{}",1),
    };


    // create serve to receive request from master
    Server::builder()
        // add struct fileuploadservice to register on server
        .add_service(FileServiceServer::new(FileUploadData))
        .add_service(MasterWorkerServer::new(worker_service))
        .serve(master_addr.parse::<SocketAddr>()?)
        .await?;
    Ok(())
}
