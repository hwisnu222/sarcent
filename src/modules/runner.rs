use std::{env, error::Error, fs, path::Path, thread, time::Duration};
use tonic::{ transport::{Channel, Server}};
use uuid::Uuid;
use crate::modules::{hash::hash_to_coordinate, repository::Repository, util::{get_storage_info, upload_stream_file}, worker::{FileUploadData, WorkerService, masterworker::{file_service_client::FileServiceClient, file_service_server::FileServiceServer, master_worker_server::MasterWorkerServer}}};
use walkdir::WalkDir;

pub async fn master_runner(repo: Repository, interval: u64, source: String, tls: bool)->Result<(), Box<dyn Error>>{
    println!("master is running, source: {}", source);
    let node = repo.get_nodes().await?;

    let root_dir = Path::new(&source);
    if let Err(e) = env::set_current_dir(&root_dir){
        eprintln!("failed change workdir to {}. Error: {}",root_dir.display(), e);
        return Err(e.into());
    }

    loop {
        // walkdir to get all file in current directory
        for entry in WalkDir::new("."){
            let entry_file = entry?;
            let path_file = entry_file.path();

            if path_file.is_file(){
                if let Some(file_name) = path_file.file_name(){
                    // hashing every file to murmur3_32
                    let hashed = hash_to_coordinate(&file_name.to_string_lossy());

                    // get server ip based on hash >= hash_key server_address
                    let mut rings = node.ring.clone();
                    let get_start_idx = rings.iter().position(|&x| hashed > x);

                    // handle if hash file get last hash vnode then the file will move in
                    // hash vnode first ring, but if vnode full storage up to ring above until
                    // get vnode with available storage
                    match get_start_idx{
                        Some(start_idx)=>{
                            rings.rotate_left(start_idx);

                            for ring_key in rings{
                                let vnodes = node.vnode_map.clone();

                                if let Some(url) =  vnodes.get(&ring_key){
                                    let limit_storage_perc  = 90.0 as f64;
                                    let protocol = if tls {"https"} else {"http"};

                                    // check available storage
                                    match get_storage_info(format!("{}://{}", protocol, url)).await{
                                        Ok(storage_info) =>{
                                            if storage_info.usage_percent < limit_storage_perc{
                                                // then use the result ip to send file
                                                println!("file {} moving to {}", path_file.to_string_lossy(), url );

                                                let protocol = if tls {"https"} else {"http"};
                                                let server_url = format!("{}://{}",protocol, url);
                                                let channel = Channel::from_shared(server_url)?.connect().await?;
                                                let mut client = FileServiceClient::new(channel);

                                                match upload_stream_file(&mut client, path_file.to_string_lossy().to_string()).await {
                                                    Ok(file_id)=>{
                                                        println!("success move file, {}", file_id);
                                                        fs::remove_file(path_file)?;
                                                    }
                                                    Err(e)=>{
                                                        eprintln!("failed upload file. Error: {}", e);
                                                    }
                                                }
                                                break;
                                            }else{
                                                println!("server can't to saving, storage is fulled");
                                                continue;
                                            }
                                        }
                                        Err(e)=>{
                                            eprintln!("failed to connect worker. Error: {}", e);
                                            continue;
                                        }
                                    }
                                }
                            }
                        },
                        None =>{
                            println!("can't found available server");
                        }
                    }
                }
            }
        };

        thread::sleep(Duration::from_secs(interval));
    }
}


pub async fn worker_runner(target: String ) -> Result<(), Box<dyn Error>>{
    println!("worker is running....\ntarget: {}/", target);

    let address = "0.0.0.0:50051".parse()?;
    let worker_service =  WorkerService{
        worker_id: format!("worker-{}", Uuid::new_v4()),
    };

    let root_dir = Path::new(&target);
    if let Err(e) = env::set_current_dir(&root_dir){
        eprintln!("failed change workdir to {}. Error: {}",root_dir.display(), e);
        return Err(e.into());
    }

    // create serve to receive request from master
    Server::builder()
        // add struct fileuploadservice to register on server
        .add_service(FileServiceServer::new(FileUploadData))
        .add_service(MasterWorkerServer::new(worker_service))
        .serve(address)
        .await?;
    Ok(())
}
