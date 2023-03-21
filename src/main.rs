mod data;
use data::*;

#[tokio::main]
async fn main() {

    sync_data().await;
    
    let mut pro_data = ProData::new().unwrap();

    let pros = pro_data.get_pros();

    for pro in pros {
        eprintln!("{:?}", pro_data.get_game(&pro).await);
    }
    
}
