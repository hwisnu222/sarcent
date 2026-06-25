use clap::{Parser};

use crate::modules::{repository::Repository, runner::{master_runner, worker_runner}, ui::{Cli, Commands, NodeAction}, util::search_files};

pub mod modules;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let repo = Repository::new().await?;

    match args.command{
        Commands::Master { interval , source, detach:_, tls} => {
            // if detach{
            //     let daemon = Daemon::new("sercent-master");
            //     daemon.run_background();
            // }

            master_runner(repo, interval, source, tls).await?;

        },
        Commands::Worker { target, detach:_} => {
            // if detach{
            //     let daemon = Daemon::new("sercent-worker");
            //     daemon.run_background();
            // }

            worker_runner(target).await?;
        },
        Commands::Node(node_args)=> {
            match node_args.action {
                NodeAction::Add { ips }  =>{
                    println!("register ip worker");
                    println!("creating 126 vnodes partition");

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
