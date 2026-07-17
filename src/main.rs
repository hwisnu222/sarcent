use clap::{Parser};
use tabled::settings::Style;
use tabled::{Table};
use tracing::{Level, info, error, debug};

use crate::repository::metadata::{MetadataRepository};
use crate::repository::migrate::MigrateDatabase;
use crate::ui::args::{Cli, Commands, NodeAction};
use crate::repository::vnode::VnodeRepository;
use crate::utils::node::server_list;
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

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    migration.migrate().await?;

    match args.command{
        Commands::Master {source, detach:_, tls:_} => {
            master::run(source).await?;
        },
        Commands::Worker { target, detach:_} => {
            worker::run(target).await?;
        },
        Commands::Node(node_args)=> {
            match node_args.action {
                NodeAction::Add { ips }  =>{
                    info!("node is registered");

                    for ip in ips{
                        repo.add_node(ip).await?;
                    }
                },
                NodeAction::List => {
                    if let Ok(servers)= server_list().await {
                        info!("Total active server: {} server", servers.len());
                        let mut table = Table::new(servers);
                        table.with(Style::modern_rounded());
                        println!("{}", table);
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
                    let mut table = Table::new(metadata);
                    table.with(Style::modern_rounded());
                    println!("{}", table);
                }
                Err(e)=>{
                    error!("{}", e);
                }
            }
        }
    }

    Ok(())
}
