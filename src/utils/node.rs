use tabled::Tabled;
use tokio::task::JoinSet;

use crate::{repository::vnode::VnodeRepository, services::storage::get_storage_info, utils::{host::get_host, size::{bytes_to_gb_string, bytes_to_percent_string}}};
use std::error::Error;

#[derive(Tabled, Clone)]
pub struct ServerInfo{
    pub address: String,
    pub total_space: String,
    pub used_space: String,
    pub availabel_space: String,
    pub usage_percent: String,

}

pub async fn server_list() -> Result<Vec<ServerInfo>, Box<dyn Error>>{
    let vnode_respository = VnodeRepository::new().await?;
    let ips = vnode_respository.get_servers().await?;
    let mut servers: Vec<ServerInfo> = vec![];
    let mut join_set = JoinSet::new();

    for ip in ips{
        join_set.spawn(async move{
            let host = get_host(&ip);
            match get_storage_info(host).await{
                Ok(res)=> ServerInfo{
                    address: ip,
                    total_space: bytes_to_gb_string(res.total_space),
                    used_space: bytes_to_gb_string( res.used_space),
                    availabel_space: bytes_to_gb_string(res.available_space),
                    usage_percent: bytes_to_percent_string(res.usage_percent)
                },
                Err(_)=> ServerInfo{
                    address: ip,
                    total_space: bytes_to_gb_string(0), 
                    used_space: bytes_to_gb_string(0),
                    availabel_space: bytes_to_gb_string(0),
                    usage_percent: bytes_to_percent_string(0_f64)
                }
            }
        });
    }

    while let Some(res) = join_set.join_next().await{
        println!("");
        if let Ok(info) = res{
            servers.push(info);
        }
    }






    // if servers.is_empty(){
    //     info!("server is empty");
    // }else{
    //     info!("Total active server: {} server", servers.len());
    //     let mut table = Table::new(servers);
    //     table.with(Style::modern_rounded());
    //     println!("{}", table);
    // }

    Ok(servers)
} 
