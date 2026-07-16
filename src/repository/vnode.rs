
use std::{error::Error};

use tokio_rusqlite::{Connection, rusqlite};

use crate::repository::base::Repository;

pub struct VnodeRepository{
    pub conn: Connection
}

impl VnodeRepository {
    pub async fn new()-> Result<Self, Box<dyn std::error::Error>>{
        let repository = Repository::new().await?;
        let conn = repository.connection;

        Ok(Self{conn})
    }
    
    pub async fn add_node(&self, server: String) -> Result<(), Box<dyn std::error::Error>>{
        let server_address = server.clone();

        self.conn.call(move |conn|{
            conn.execute("INSERT OR REPLACE INTO vnodes (server_address) VALUES(?1)", rusqlite::params![server_address])
        }).await?;
        Ok(())
    }

    // remove node by server_address
    pub async fn remove_node(&self, server_address: String) -> Result<usize, Box<dyn Error>>{
        let node_removed = self.conn.call(move |conn|{
            conn.execute("DELETE FROM vnodes WHERE server_address = ?1", [server_address])
        }).await?;

        Ok(node_removed)
    }

    pub async fn get_servers(&self) -> Result<Vec<String>, Box<dyn Error>>{
        let servers = self.conn.call(|conn| -> Result<Vec<String>, tokio_rusqlite::Error>{
            let mut stmt = conn.prepare("SELECT DISTINCT server_address FROM vnodes ORDER BY created_at ASC")?;

            let mut server_list = Vec::new();
            let mut rows = stmt.query([])?;

            while let Some(row) = rows.next()?{
                let server = row.get(0)?;
                server_list.push(server);
            };

            Ok(server_list)
        }).await?;
        Ok(servers)
    }

    
}
