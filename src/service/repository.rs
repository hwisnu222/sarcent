use std::{collections::HashMap, error::Error};

use tokio_rusqlite::{Connection, rusqlite};

use crate::service::hash::hash_to_coordinate;

pub struct HashRing{
    pub ring: Vec<i64>,
    pub vnode_map: HashMap<i64, String>
}

pub struct Repository{
    pub conn: Connection
}

impl Repository {
    pub async fn new()-> Result<Self, Box<dyn std::error::Error>>{
        let path_db = "sercent.db";
        let conn = Connection::open(path_db).await?;

        conn.call(|conn|{
            conn.execute("CREATE TABLE IF NOT EXISTS vnodes (
                hash_coordinate INTEGER PRIMARY KEY,
                server_address TEXT NOT NULL,
                server_key TEXT NOT NULL
            )", [])
        }).await?;

        Ok(Self{conn})
    }

    pub async fn add_node(&self, server: String) -> Result<(), Box<dyn std::error::Error>>{
        // partition every server to 126 partition node
        let max_node: i32 = 126;
        for i in 0..max_node{
            let vnode = format!("{}#{}", server, i);
            let hash_coordinate =  hash_to_coordinate(&vnode);
            let server_address = server.clone();

            self.conn.call(move |conn|{
                conn.execute("INSERT OR REPLACE INTO vnodes (hash_coordinate, server_address, server_key) VALUES(?1,?2, ?3)", rusqlite::params![hash_coordinate, server_address, vnode])
            }).await?;
        }
        Ok(())
    }

    pub async fn get_nodes(&self) -> Result<HashRing, Box<dyn Error>>{
        let data = self.conn.call( |conn| -> Result<HashRing, tokio_rusqlite::Error>{
            let mut stmt = conn.prepare("SELECT hash_coordinate,server_address FROM vnodes ORDER BY hash_coordinate ASC")?;
            
            let mut ring = Vec::new();
            let mut vnode_map = HashMap::new();
            let mut rows = stmt.query([])?;

            while let Some(row) = rows.next()?{
                let hash: i64 = row.get(0)?;
                let address: String = row.get(1)?;

                ring.push(hash);
                vnode_map.insert(hash, address);
            }

            Ok(HashRing{ring, vnode_map})
        }).await?;

        Ok(data)
    }

    pub async fn get_servers(&self) -> Result<Vec<String>, Box<dyn Error>>{
        let servers = self.conn.call(|conn| -> Result<Vec<String>, tokio_rusqlite::Error>{
            let mut stmt = conn.prepare("SELECT DISTINCT server_address FROM vnodes ORDER BY server_address ASC")?;

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
