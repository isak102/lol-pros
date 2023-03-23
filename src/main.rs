mod data;
use data::*;

#[tokio::main]
async fn main() {
    eprintln!(
        "sync_data(): {:?}",
        sync_data::sync_data::sync_summoner_ids().await
    );

    let mut pro_data: ProData = match ProData::new() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    eprintln!("Getting pros...");
    let pros = &pro_data.get_pros();
    eprintln!("Pros: {:?}", pros);

    for pro in pros {
        eprintln!("Game: {:?}", pro_data.get_game(pro).await);
    }

    eprintln!(
        "\nFound {} game(s) with {} pro(s) in total",
        pro_data.games_count(),
        pro_data.pros_in_game_count()
    );
}
