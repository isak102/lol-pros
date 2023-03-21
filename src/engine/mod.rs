use std::collections::HashMap;

use riven::models::spectator_v4::*;
use riven::RiotApi;
use super::data::*;

struct Game {
    game_info: CurrentGameInfo,
    pros: Vec<Pro>,
}

// TODO: make public function that takes in a hash map of players and loops through each pro and
// finds a game for him.
pub fn run(pro_map: HashMap<String, Pro>) -> Result<(), String> {

    let riot_api = RiotApi::new("API KEY");

    for pro in pro_map.values() {
        if let Some(account_id) = pro.get_account_id() {
            println!("id: {}", account_id);
        } else {
            // return Err("Couldn't find account_ID for pro".to_string());
        }
    }
   
    Ok(())
}
