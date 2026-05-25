use std::net::SocketAddr;

use clap::{Parser, ValueEnum};
use tonic::transport::Server;

use crate::service::worker::{WorkerService, masterworker::{self, StorageRequest, master_worker_server::MasterWorkerServer}};

pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, ValueEnum)]
pub enum Mode{
    Master,
    Worker,
}

#[derive(Debug, Parser)]
#[command(version, about="sercent - file server worker")]
struct Args{
    #[arg(short, long,value_enum, default_value_t=Mode::Worker)]
    mode: Mode,

    #[arg(short, long, default_value="0.0.0.0:50051")]
    address: String,

    #[arg(short, long, default_value="worker-01")]
    worker_id: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.mode {
        Mode::Master => {
            println!("master is running");
            // list all worker
            let workers = vec!["http://localhost:50051"];

            for worker_addr in workers{
                println!("Connecting to {}", worker_addr);

                match masterworker::master_worker_client::MasterWorkerClient::connect(worker_addr).await{
                    Ok(mut client)=> {
                        println!("connected");

                        let request = tonic::Request::new(StorageRequest{
                            worker_id: args.worker_id.clone(),
                        });

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
        },
        Mode::Worker =>{
            println!("worker is running...");
            let worker_service =  WorkerService{
                worker_id: "worker-01".to_string(),
            };

            Server::builder()
                .add_service(MasterWorkerServer::new(worker_service))
                .serve(args.address.parse::<SocketAddr>()?)
                .await?;
        }
    }

    println!("{:?}", args.mode);
    Ok(())
}
