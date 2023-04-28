mod api;
mod args;
mod pro_data;
mod ui;

use std::process;

use clap::Parser;
use pro_data::*;
use riven::reqwest::StatusCode;
use yansi::Paint;

pub struct Config {
    pub pro_file_path: String, // FIXME: turn this into a path
}

#[tokio::main]
async fn main() {
    let args = args::Args::parse();
    let disable_colors;

    // disable color if CLICOLOR is set to 0
    if let Ok(true) = std::env::var("CLICOLOR").map(|v| v == "0") {
        disable_colors = true;
    } else {
        disable_colors = args.disable_colors;
    }
    if disable_colors {
        Paint::disable();
    }

    let c = Config {
        pro_file_path: args.pro_file_path,
    };

    match args.command {
        Some(cmd) => match cmd {
            args::Command::Sync {} => {
                pro_data::io::sync_summoner_ids(&c)
                    .await
                    .unwrap_or_else(|err| {
                        eprintln!("Error when syncing summoner IDs: {}", err);
                        process::exit(1);
                    });
                eprintln!("Done syncing summoner IDs");
                process::exit(0);
            }
        },
        None => {}
    }

    eprintln!("Getting pros...");
    let mut pro_data = ProData::load(&c).await.unwrap_or_else(|err| {
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
        ui::table::print(&game)
            .await
            .expect("printing should succeed");
    }

    println!(
        "\nFound {} game(s) with {} pro(s) in total. {} pro(s) exist in the database.",
        pro_data.games_count(),
        pro_data.pros_in_game_count(),
        pro_data.pros_count(),
    );
}
