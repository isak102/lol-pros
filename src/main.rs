mod api_key;
mod config;
mod pro_data;

use std::env;
use std::process;

use config::Config;
use pro_data::*;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::parse(&args).unwrap_or_else(|err| {
        eprintln!("Error when parsing configuration: {err}");
        process::exit(1);
    });

    eprintln!("Getting pros...");
    let mut pro_data = ProData::load(&config).await.unwrap_or_else(|err| {
        eprintln!("Error when loading ProData: {err}");
        process::exit(1);
    });

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
