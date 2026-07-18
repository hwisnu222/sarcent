use std::process;

use clap::{Parser};
use tracing::{Level, info, error, debug};

use crate::repository::metadata::{MetadataRepository};
use crate::repository::migrate::MigrateDatabase;
use crate::ui::args::{Cli, Commands, NodeAction};
use crate::repository::vnode::VnodeRepository;
use crate::utils::node::server_list;
use crate::utils::size::bytes_to_gb_string;
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
    let migration = MigrateDatabase::new().await?;

    let level_tracing = if args.verbose {Level::DEBUG} else {Level::INFO};

    tracing_subscriber::fmt()
        .with_max_level(level_tracing)
        .init();

    // run all migrations file 
    migration.migrate().await?;

    match args.command{
        Commands::Master {source} => {
            master::run(source).await?;
        },
        Commands::Worker { target, port} => {
            worker::run(target, port).await?;
        },
        Commands::Node(node_args)=> {
            match node_args.action {
                NodeAction::Add { ips }  =>{
                    for ip in ips{
                        repo.add_node(ip).await?;
                    }

                    info!("node has been registered");
                },
                NodeAction::List => {
                    if let Ok(servers)= server_list().await {
                        info!("Total active server: {} server", servers.len());
                        // let mut table = Table::new(servers);
                        // table.with(Style::modern_rounded());
                        // println!("{}", table);

                        servers.iter().for_each(|server|{
                            println!("[{:?}] [{}] used_space: {}, available: {}",server.status, server.address, server.used_space, server.available_space);
                        });
                    }
                },
                NodeAction::Remove{ip}=>{
                    match repo.remove_node(ip.clone()).await{
                        Ok(_) => {
                            info!("{} server is deleted", ip);
                        }
                        Err(e) => {
                            debug!("{}", e);
                            error!("failed remove server node");
                        }
                   }
                },
            }
        },
        Commands::Search{filename} => {
            let metadata_respository = MetadataRepository::new().await?;

            match metadata_respository.search_by_name(filename).await{
                Ok(metadata)=>{
                    if metadata.len() < 1 {
                        info!("there are not files found!");
                        process::exit(1);
                    }

                    metadata.iter().for_each(|item|{
                        println!("[{}] {} {}", item.server_address, bytes_to_gb_string(item.size_bytes as u64), item.name);
                    });
                }
                Err(e)=>{
                    error!("{}", e);
                }
            }
        }
    }

    Ok(())
}
