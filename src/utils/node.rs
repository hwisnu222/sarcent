use tokio::task::JoinSet;

use crate::{repository::vnode::VnodeRepository, services::storage::get_storage_info, utils::{host::get_host, size::{bytes_to_gb_string, bytes_to_percent_string}}};
use std::error::Error;

#[derive(Debug, Clone)]
pub enum Status{
    Online,
    Offline
}

#[derive(Clone)]
pub struct ServerInfo{
    pub address: String,
    pub total_space: String,
    pub used_space: String,
    pub available_space: String,
    pub usage_percent: String,
    pub status: Status
}

// implementation display for enum
// because print need {:?} to display data
// impl fmt::Display for Status {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>)-> fmt::Result{
//         match self{
//             Status::Online => write!(f, "ONLINE"),
//             Status::Offline => write!(f, "OFFLINE"),
//         }
//     }
// }

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
                    available_space: bytes_to_gb_string(res.available_space),
                    usage_percent: bytes_to_percent_string(res.usage_percent),
                    status: Status::Online
                },
                Err(_)=> ServerInfo{
                    address: ip,
                    total_space: bytes_to_gb_string(0), 
                    used_space: bytes_to_gb_string(0),
                    available_space: bytes_to_gb_string(0),
                    usage_percent: bytes_to_percent_string(0_f64),
                    status: Status::Offline,
                }
            }
        });
    }

    while let Some(res) = join_set.join_next().await{
        // take all data because when server return error
        // that is a signal a server is offline
        if let Ok(info) = res{
            servers.push(info);
        }
    }

    Ok(servers)
} 
