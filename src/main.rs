mod data;

#[tokio::main]
async fn main() {
    
    let pros = data::get_pros();
    match pros {
        Err(e) => println!("Error getting pros: {}", e),
        Ok(val) => for key in val.keys() {
            println!("Name: {}", key);
        }
    }
    
}
