mod data;
use data::*;

#[tokio::main]
async fn main() {
    
    let mut pro_data = ProData::new().unwrap();

    let game = pro_data.get_game("Emtest").await;

    println!("{:?}", game);
    println!("{:?}", pro_data);
}
