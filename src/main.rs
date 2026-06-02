use clap::{Parser};

use crate::modules::{repository::{Repository}, runner::{master_runner, worker_runner}, ui::{Cli, Command, NodeAction}};

pub mod modules;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let repo = Repository::new().await?;

    match args.command{
        Command::Master { interval , source, detach:_, tls} => {
            // if detach{
            //     let daemon = Daemon::new("sercent-master");
            //     daemon.run_background();
            // }

            master_runner(repo, interval, source, tls).await?;

        },
        Command::Worker { target, detach:_} => {
            // if detach{
            //     let daemon = Daemon::new("sercent-worker");
            //     daemon.run_background();
            // }

            worker_runner(target).await?;
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

                }
                
            }
        }
    }

    Ok(())
}
