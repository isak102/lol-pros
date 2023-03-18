mod data;

#[tokio::main]
async fn main() {
    let pros = data::get_pros();
    match pros {
        Err(e) => println!("Error getting pros: {}", e),
        Ok(result) => {
            for val in result.values() {
                println!("{}", val);
            }
        }
    }
}
