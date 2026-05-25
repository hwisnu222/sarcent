pub mod masterworker {
    tonic::include_proto!("masterworker");
}

use masterworker::master_worker_server::{MasterWorker};
use masterworker::{StorageRequest, StorageResponse};
use sysinfo::Disks;

#[derive(Clone)]
pub struct WorkerService{
    pub worker_id: String
}

#[tonic::async_trait]
impl MasterWorker for WorkerService{
    async fn get_storage_info(&self, _request: tonic::Request<StorageRequest>)-> Result<tonic::Response<StorageResponse>, tonic::Status>{
        // ambil struct Disks untuk mendapatkan informasi disk
        // jika kamu ingin mendapatkan CPU atau ram gunakan System
        // Ini untuk CPU, RAM, dll (seperti biasa)
        // let mut sys = System::new_all();
        // sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        

        let disk = disks.first().ok_or_else(|| {tonic::Status::internal("no disk found")})?;
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total - available;

        let response = StorageResponse{
            worker_id: self.worker_id.clone(),
            total_space: total,
            used_space: used,
            available_space: available,
            usage_percent: if total > 0 {(used as f64 / total as f64)*100.0} else {0.0},
        };
        Ok(tonic::Response::new(response))
    }

    // fn monitor(&self, _request: tonic::Request<MasterCommand>)-> Result<tonic::Response<tonic::Streaming<WorkerStatus>>, tonic::Status>{
    //     let response = 
    //     Ok(tonic::Response::new())
    // }
    
}
