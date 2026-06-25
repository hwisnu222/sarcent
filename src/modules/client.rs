use std::{collections::HashMap, error::Error, time::Duration};

use tonic::transport::{Channel, Endpoint};

use crate::modules::{repository::Repository, worker::masterworker::file_service_client::FileServiceClient};

pub struct FileServiceClusterClient {
    pub pool: HashMap<String, FileServiceClient<Channel>>
}

impl FileServiceClusterClient{
    pub async fn init() -> Result<Self, Box<dyn Error>>{
        let mut pool = HashMap::new();
        let repo = Repository::new().await?;
        let servers = repo.get_servers().await?;

        for server in servers.into_iter(){
            println!("initializing server... {}", server);
            let host = format!("http://{}", server);

            // timeout server when over limit 2 seconds
            // continue to next server
            let endpoint = Endpoint::from_shared(host.clone())?
                .timeout(Duration::from_secs(5))
                .connect_timeout(Duration::from_secs(5))
                .connect_lazy(); // limit handshake 2 seconds
            let client = FileServiceClient::new(endpoint);
            pool.insert(server, client);

            // match FileServiceClient::connect(endpoint).await{
            //     Ok(client) => {
            //         println!("success connect to {}", server);
            //         pool.insert(server, client);
            //     }
            //     Err(e)=>{
            //         eprintln!("failed connect to {}, Error: {}", server, e);
            //     }
            // }
        }
        Ok(Self{pool})
    }

    pub fn get_client(&self, server_address: &str) -> Option<FileServiceClient<Channel>>{
        self.pool.get(server_address).cloned()
    }
}
