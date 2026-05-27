use std::{net::SocketAddr, thread, time::Duration};

use clap::{Parser};
use tonic::transport::Server;

use crate::service::{repository::Repository, ui::{Cli, Command, NodeAction}, worker::{WorkerService, masterworker::{self, StorageRequest, master_worker_server::MasterWorkerServer}}};

pub mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let repo = Repository::new().await?;

    match args.command{
        Command::Master { interval , target} => {
            println!("master is running, target: {}", target);
            // list all worker
            let servers = repo.get_servers().await?;


            loop {
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


        },
        Command::Worker { master_addr, target} => {
            println!("worker is running.... target: {}", target);
            let worker_service =  WorkerService{
                worker_id: "worker-01".to_string(),
            };

            Server::builder()
                .add_service(MasterWorkerServer::new(worker_service))
                .serve(master_addr.parse::<SocketAddr>()?)
                .await?;

        },
        Command::Node(node_args)=> {
            match node_args.action {
                NodeAction::Add { ips }  =>{
                    println!("register ip worker");

                    for ip in ips{
                        println!("creating 126 vnodes partition");
                        repo.add_node(ip).await?;
                    }
                },
                NodeAction::List => {
                   let servers = repo.get_servers().await?;

                   if servers.is_empty(){
                       println!("server is empty");
                   }else{
                       println!("Total active server: {} server", servers.len());
                       for (index, ip) in servers.iter().enumerate(){
                           println!("{}.  [{}]", index+1, ip);
                       }
                   }
                }
                
            }
        }
    }

    Ok(())
}
