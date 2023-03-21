use csv::ReaderBuilder;
use riven::consts::PlatformRoute;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::vec;
use std::{collections::HashMap, io::Result};

use riven::models::spectator_v4::*;
use riven::RiotApi;

const PRO_FILE: &str = "/home/isak102/.local/share/pros.csv";

pub type AccountID = String;
pub type SummonerName = String;
pub type TeamShort = String;
pub type TeamFull = String;

#[derive(Debug)]
pub struct ProData {
    pros: HashMap<SummonerName, Pro>,
    games: Vec<Game>, // TODO: store each game with a tuple of a vector of summoner names and
                      // games
}

impl ProData {
    pub fn new() -> Result<ProData> {
        let file = File::open(PRO_FILE)?;
        let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);

        let mut pros = HashMap::new();

        for result in reader.records() {
            let record = result?;

            let player_name: String = record[0].to_string();
            let team_short_name: TeamShort = record[1].to_string();
            let team_full_name: TeamFull = record[2].to_string();
            let summoner_name: SummonerName = record[3].to_string();
            let account_id: AccountID = record[4].to_string();

            let team = Team::new(team_short_name, team_full_name);
            let pro = Pro::new(player_name, team, summoner_name.clone(), account_id);

            pros.insert(summoner_name, pro);
        }

        Ok(ProData {
            pros: pros,
            games: Vec::new(),
        })
    }

    pub async fn get_game<'a>(&'a mut self, pro_summoner_name: &str) -> Result<Option<&'a Game>> {
        let riot_api = RiotApi::new("RGAPI-56173203-f7b4-4382-85e1-6b2869c1c4f5"); // TODO: make Config struct, should include ProMap ??

        let pro = match self.pros.get_mut(pro_summoner_name) {
            Some(pro) => pro,
            None => {
                return Err(Error::new(
                    // TODO: make custom error
                    ErrorKind::Other,
                    format!("{} doesn't exist in ProData", pro_summoner_name),
                ));
            }
        };

        let account_id: &AccountID = match &pro.account_id {
            Some(id) => id,
            None => {
                return Err(Error::new(
                    // TODO: make custom error
                    ErrorKind::Other,
                    format!(
                        "{} {} has no account ID",
                        pro.team.short_name, pro.player_name
                    ),
                ));
            }
        };

        let game_info = match riot_api
            .spectator_v4()
            .get_current_game_info_by_summoner(PlatformRoute::EUW1, account_id)
            .await
        {
            Ok(game) => match game {
                Some(game) => game,
                None => return Ok(None),
            },
            Err(_) => {
                return Err(Error::new(
                    // TODO: make custom error
                    ErrorKind::Other,
                    format!(
                        "Error when finding game for {} {}",
                        pro.team.short_name, pro.player_name
                    ),
                ));
            }
        };

        pro.game_found = true;

        let pro_players = self.find_pros_in_game(&game_info.participants);

        let game = Game {
            game_info,
            pro_players: pro_players,
        };
        self.games.push(game);

        Ok(Some(&self.games.last().expect(
            "We just pushed game so this should be Some(Game)",
        )))
    }

    fn find_pros_in_game(&mut self, summoners: &Vec<CurrentGameParticipant>) -> Vec<SummonerName> {
        let mut pros_in_this_game: Vec<SummonerName> = Vec::new();
        for summoner in summoners {
            let summoner_name = &summoner.summoner_name;

            match self.pros.get_mut(summoner_name) {
                Some(pro) => {
                    pro.game_found = true;
                    pros_in_this_game.push(summoner_name.clone());
                }

                None => {}
            }
        }

        pros_in_this_game
    }
}

#[derive(Debug)]
pub struct Team {
    pub short_name: String,
    pub full_name: String,
}

impl Team {
    pub fn new(short_name: String, full_name: String) -> Team {
        Team {
            short_name,
            full_name,
        }
    }
}

#[derive(Debug)]
pub struct Game {
    game_info: CurrentGameInfo,
    pro_players: Vec<SummonerName>,
}

#[derive(Debug)]
pub struct Pro {
    pub player_name: String,
    pub team: Team,
    pub summoner_name: String,
    account_id: Option<String>,
    game_found: bool,
}

impl Pro {
    fn new(player_name: String, team: Team, summoner_name: String, account_id_str: String) -> Pro {
        let mut account_id = None;
        if !account_id_str.is_empty() {
            account_id = Some(account_id_str);
        }

        Pro {
            player_name,
            team,
            summoner_name,
            account_id,
            game_found: false,
        }
    }
}

impl std::fmt::Display for Pro {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {} - {}",
            self.team.short_name, self.player_name, self.summoner_name
        )
    }
}
