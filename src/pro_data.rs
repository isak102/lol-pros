use riven::consts::PlatformRoute;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Write;
use std::ops::Index;

use std::rc::Rc;

use riven::models::spectator_v4::*;
use riven::{RiotApi, RiotApiError};

use self::io::load_pros;
pub use self::pro_game::*;
use crate::api_key;
use crate::config::Config;

pub mod io;
mod pro_game;

pub type PlayerName = String;
pub type SummonerID = String;
pub type SummonerName = String;
pub type TeamShort = String;
pub type TeamFull = String;

#[derive(Debug, Clone)]
pub struct Pro {
    player_name: String,
    team: Team,
    summoner_name: String,
    summoner_id: Option<String>,
}

#[derive(Debug, Clone)]
struct Team {
    short_name: String,
    full_name: String,
}

#[derive(Debug)]
pub struct ProData {
    pros: HashMap<SummonerName, Rc<Pro>>,
    games: Vec<Rc<ProGame>>,
    pros_in_game: HashMap<SummonerName, Rc<ProGame>>,
}

impl Pro {
    fn new(player_name: String, team: Team, summoner_name: String, summoner_id_str: String) -> Pro {
        let mut summoner_id = None;
        if !summoner_id_str.is_empty() {
            summoner_id = Some(summoner_id_str);
        }

        Pro {
            player_name,
            team,
            summoner_name,
            summoner_id,
        }
    }
}

impl std::fmt::Display for Pro {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.team.short_name, self.player_name)
    }
}

impl Team {
    fn new(short_name: String, full_name: String) -> Team {
        Team {
            short_name: short_name.to_uppercase(),
            full_name,
        }
    }
}

impl ProData {
    pub async fn load(config: &Config) -> Result<ProData, Box<dyn Error>> {
        let pros = load_pros(config).await?;

        Ok(ProData {
            pros: pros,
            games: Vec::new(),
            pros_in_game: HashMap::new(),
        })
    }

    // TODO: find way to return Vec<&Pro>
    pub fn get_pros(&self) -> Vec<Rc<Pro>> {
        let mut result = Vec::new();
        for (_, val) in &self.pros {
            result.push(Rc::clone(&val));
        }
        result
    }

    pub async fn fetch_game(
        &mut self,
        pro: &Pro,
    ) -> std::result::Result<Option<Rc<ProGame>>, RiotApiError> {
        let riot_api = RiotApi::new(api_key::API_KEY);

        let summoner_id: &SummonerID = match &pro.summoner_id {
            Some(id) => id,
            None => {
                // TODO: add better error handling here
                panic!("Summoner had no summoner_id, call sync_summoner_ids before fetching games")
            }
        };

        /* If this pro already is in a found game then we return that game instantly */
        if let Some(game) = self.pros_in_game.get(&pro.summoner_name) {
            return Ok(Some(Rc::clone(&game)));
        }

        // FIXME: filter out all games that are not ranked games

        let game_info = match riot_api
            .spectator_v4()
            .get_current_game_info_by_summoner(PlatformRoute::EUW1, summoner_id)
            .await?
        {
            Some(g) => g,
            None => return Ok(None),
        };

        let pro_players = self.find_pros_in_game(&game_info);

        let game = Rc::new(ProGame {
            game_info,
            pro_players: pro_players,
        });

        /* Insert each pro player in this game into the hashmap of pro_players that are in game. */
        for pro_player in &game.pro_players {
            self.pros_in_game
                .insert(pro_player.summoner_name.clone(), Rc::clone(&game));
        }

        let game_clone = Rc::clone(&game);
        self.games.push(game);

        Ok(Some(game_clone))
    }

    fn find_pros_in_game(&self, game_info: &CurrentGameInfo) -> Vec<Rc<Pro>> {
        let mut pros_in_this_game: Vec<Rc<Pro>> = Vec::new();
        let summoners: &Vec<CurrentGameParticipant> = &game_info.participants;
        for summoner in summoners {
            let summoner_name = &summoner.summoner_name;

            match self.pros.get(summoner_name) {
                Some(pro) => {
                    pros_in_this_game.push(Rc::clone(pro));
                }

                None => {}
            }
        }

        pros_in_this_game
    }

    pub fn games_count(&self) -> usize {
        self.games.len()
    }

    pub fn pros_in_game_count(&self) -> usize {
        self.pros_in_game.len()
    }
}
