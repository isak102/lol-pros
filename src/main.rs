mod api;
mod config;
mod pro_data;
mod ui;

use std::process;

use config::Config;
use pro_data::*;
use riven::reqwest::StatusCode;

#[tokio::main]
async fn main() {
    let config = Config::parse().unwrap_or_else(|err| {
        eprintln!("Error when parsing configuration: {err}");
        process::exit(1);
    });

    eprintln!("Getting pros...");
    let mut pro_data = ProData::load(&config).await.unwrap_or_else(|err| {
        eprintln!("Error when loading pro data: {err}");
        process::exit(1);
    });

    let pros = &pro_data.get_pros();
    for pro in pros {
        if pro_data.is_in_game(pro) {
            continue;
        }
        let game = match pro_data.fetch_game(pro).await {
            Err(e) => {
                if e.status_code() == Some(StatusCode::FORBIDDEN) {
                    eprintln!("ERROR: 403 received, probably due to bad API key");
                    std::process::exit(1);
                }
                eprintln!("Error when fetching game for {pro}: {}", e);
                continue;
            }
            Ok(result) => match result {
                None => {
                    println!("<{pro}> offline...");
                    continue;
                }
                Some(g) => g,
            },
        };
        ui::table::print(&game).expect("printing should succeed");
    }

    println!(
        "\nFound {} game(s) with {} pro(s) in total",
        pro_data.games_count(),
        pro_data.pros_in_game_count()
    );
}
