use core::panic;
use csv::ReaderBuilder;
use riven::consts::PlatformRoute;
use std::fmt::Write;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::ops::Index;
use std::{collections::HashMap, io::Result};

use std::rc::Rc;

use riven::models::spectator_v4::*;
use riven::RiotApi;

use crate::api_key;

const PRO_FILE: &str = "/home/isak102/.local/share/pros.csv";

pub type PlayerName = String;
pub type SummonerID = String;
pub type SummonerName = String;
pub type TeamShort = String;
pub type TeamFull = String;

pub mod io;

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

#[derive(Debug, Clone)]
pub struct ProGame {
    // TODO: implement Display
    game_info: CurrentGameInfo,
    pro_players: Vec<Rc<Pro>>,
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

impl std::fmt::Display for ProGame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut blue_team: Vec<&CurrentGameParticipant> = Vec::new();
        let mut red_team: Vec<&CurrentGameParticipant> = Vec::new();

        for participant in &self.game_info.participants {
            match &participant.team_id {
                riven::consts::Team::BLUE => blue_team.push(&participant),
                riven::consts::Team::RED => red_team.push(&participant),
                _ => panic!("Team should be either BLUE or RED."),
            }
        }

        let mut output = String::new();

        for i in 0..5 {
            let blue_player: &CurrentGameParticipant = blue_team.index(i);
            let red_player: &CurrentGameParticipant = red_team.index(i);

            let (blue_is_pro, blue_pro_name) = {
                let pro = self.get_pro(&blue_player.summoner_name);
                let is_pro = pro.is_some();
                let mut pro_name = String::new();

                if is_pro {
                    let pro = pro.unwrap();
                    write!(pro_name, "{} {}", pro.team.short_name, pro.player_name).unwrap();
                }

                (is_pro, pro_name)
            };

            // TODO: make anonymous function and combine this code below with the one above
            let (red_is_pro, red_pro_name) = {
                let pro = self.get_pro(&red_player.summoner_name);
                let is_pro = pro.is_some();
                let mut pro_name = String::new();

                if is_pro {
                    let pro = pro.unwrap();
                    write!(pro_name, "{} {}", pro.team.short_name, pro.player_name).unwrap();
                }

                (is_pro, pro_name)
            };

            write!(
                output,
                "{0: <40} {1}",
                participant_to_string(blue_player, (blue_is_pro, &blue_pro_name)),
                participant_to_string(red_player, (red_is_pro, &red_pro_name)),
            )?;

            /* dont append newline if we are on the last line */
            if i != 4 {
                output.push('\n');
            }
        }

        // TODO: extract into function
        write!(
            output,
            "\n\nBanned champions: {:?}",
            &self.game_info.banned_champions
        )
        .unwrap();

        write!(f, "{}", output)?;
        Ok(())
    }
}

fn participant_to_string(participant: &CurrentGameParticipant, is_pro: (bool, &str)) -> String {
    let mut result = String::new();
    if let (true, pro_name) = is_pro {
        write!(result, "<{}> ", pro_name).unwrap();
    }

    write!(
        result,
        "{} [{}]",
        participant.champion_id.name().unwrap(),
        participant.summoner_name,
    )
    .unwrap();

    result
}

impl ProGame {
    fn get_pro(&self, summoner_name: &SummonerName) -> Option<&Pro> {
        for pro in &self.pro_players {
            if pro.as_ref().summoner_name.eq(summoner_name) {
                return Some(pro.as_ref());
            }
        }
        None
    }
}

impl Team {
    fn new(short_name: String, full_name: String) -> Team {
        Team {
            short_name,
            full_name,
        }
    }
}

impl ProData {
    pub fn new() -> Result<ProData> {
        let file = File::open(PRO_FILE)?;
        let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);

        let mut pros = HashMap::new();

        for result in reader.records() {
            let record = result?;

            let player_name: PlayerName = record[0].to_string();
            let team_short_name: TeamShort = record[1].to_string();
            let team_full_name: TeamFull = record[2].to_string();
            let summoner_name: SummonerName = record[3].to_string();
            let summoner_id: SummonerID = record[4].to_string();

            let team = Team::new(team_short_name, team_full_name);
            let pro = Pro::new(player_name, team, summoner_name.clone(), summoner_id);

            pros.insert(summoner_name, Rc::new(pro));
        }

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

    pub async fn get_game(&mut self, pro: &Pro) -> Result<Option<Rc<ProGame>>> {
        let riot_api = RiotApi::new(api_key::API_KEY);

        let summoner_id: &SummonerID = match &pro.summoner_id {
            Some(id) => id,
            None => {
                return Err(Error::new(
                    // TODO: make custom error
                    ErrorKind::Other,
                    format!(
                        "{} {} has no summoner ID",
                        pro.team.short_name, pro.player_name
                    ),
                ));
            }
        };

        /* If this pro already is in a found game then we return that game instantly */
        if let Some(game) = self.pros_in_game.get(&pro.summoner_name) {
            return Ok(Some(Rc::clone(&game)));
        }

        let game_info = match riot_api
            .spectator_v4()
            .get_current_game_info_by_summoner(PlatformRoute::EUW1, summoner_id)
            .await
        {
            Ok(game) => match game {
                Some(game) => game,
                None => return Ok(None),
            },
            Err(e) => {
                eprint!("{}", e);
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