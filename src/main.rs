mod data;
use data::*;

#[tokio::main]
async fn main() {
    data::sync_data::sync_summoner_ids().await.unwrap();

    let mut pro_data: ProData = match ProData::new() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    eprintln!("Getting pros...");
    let pros = &pro_data.get_pros();

    for pro in pros {
        let game = match pro_data.get_game(pro).await.unwrap() {
            Some(g) => g,
            None => {
                println!("...");
                continue;
            }
        };
        println!("{}\n...", game);
    }

    println!(
        "\nFound {} game(s) with {} pro(s) in total",
        pro_data.games_count(),
        pro_data.pros_in_game_count()
    );
}
