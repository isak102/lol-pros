mod data;

use std::process::*;
use data::*;

#[tokio::main]
async fn main() {

    eprintln!("sync_data(): {:?}", sync_data().await);
    
    let mut pro_data: ProData = match ProData::new() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };

    eprintln!("Getting pros...");
    let pros = pro_data.get_pros();
    eprintln!("Pros: {:?}", pros);

    for pro_ptr in pros {
        eprintln!("Game: {:?}", pro_data.get_game(pro_ptr).await);
    }
    
    eprintln!("pro_data: {:?}", pro_data);
    
}
