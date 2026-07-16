use clap::{Parser};

use crate::ui::args::{Cli, Commands, NodeAction};
use crate::repository::vnode::VnodeRepository;
use crate::services::file::search_files;
use crate::{servers::{master, worker}};

pub mod utils;
pub mod repository;
pub mod services;
pub mod servers;
pub mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let repo = VnodeRepository::new().await?;

    match args.command{
        Commands::Master {source, detach:_, tls} => {
            master::run(source, tls).await?;
        },
        Commands::Worker { target, detach:_} => {
            worker::run(target).await?;
        },
        Commands::Node(node_args)=> {
            match node_args.action {
                NodeAction::Add { ips }  =>{
                    println!("register node");

                    for ip in ips{
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
                           println!("{}.  {}", index+1, ip);
                       }
                   }
                },
                NodeAction::Remove{ip}=>{
                    match repo.remove_node(ip.clone()).await{
                        Ok(_) => {
                            println!("server node with {} address is deleted", ip);
                        }
                        Err(e) => {
                            eprintln!("failed remove server node. Error: {}", e);
                        }
                   }
                },
                NodeAction::Search { filename } =>{
                    match search_files(filename.to_string()).await {
                        Ok(responses)=>{
                            for response in responses{
                                for file in response.files{
                                    println!("{}", file.file_name);
                                }
                            }
                        }
                        Err(e)=>{
                            eprintln!("Error: {}", e);
                        }
                   }
                }
            }
        }
    }

    Ok(())
}
