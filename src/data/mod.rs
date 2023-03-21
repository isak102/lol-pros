use csv::{ReaderBuilder, WriterBuilder};
use riven::consts::PlatformRoute;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::{collections::HashMap, io::Result};

use riven::models::spectator_v4::*;
use riven::RiotApi;

const PRO_FILE: &str = "/home/isak102/.local/share/pros.csv";
const API_KEY: &str = "RGAPI-ff208cbe-ae4d-4983-9ec4-8b291da869a5";

pub type SummonerID = String;
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
            let summoner_id: SummonerID = record[4].to_string();

            let team = Team::new(team_short_name, team_full_name);
            let pro = Pro::new(player_name, team, summoner_name.clone(), summoner_id);

            pros.insert(summoner_name, pro);
        }

        Ok(ProData {
            pros: pros,
            games: Vec::new(),
        })
    }

    pub async fn get_game<'a>(&'a mut self, pro_summoner_name: &str) -> Result<Option<&'a Game>> {
        let riot_api = RiotApi::new(API_KEY);

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

fn get_api_key() -> String {
    /* TODO: fix better error handling */
    let file = File::open("/home/isak102/.local/share/RGAPI.txt")
        .expect("This file is hardcoded and should exist");
    let mut reader = BufReader::new(file);
    let mut api_key = String::new();

    reader
        .read_line(&mut api_key)
        .expect("This file needs to have the API key on line 1");

    api_key
}

async fn get_summoner_id(summoner_name: &SummonerName) -> Result<SummonerID> {
    let riot_api = RiotApi::new(API_KEY);

    let summoner = match riot_api
        .summoner_v4()
        .get_by_summoner_name(PlatformRoute::EUW1, summoner_name)
        .await
    {
        Ok(s) => match s {
            Some(summoner) => summoner,
            None => return Err(Error::new(ErrorKind::Other, "Summoner not found")),
        },
        Err(_) => return Err(Error::new(ErrorKind::Other, "Error getting summoner info")),
    };

    Ok(summoner.id)
}

pub async fn sync_data() -> Result<()> {
    let old_file = File::open(PRO_FILE)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(old_file);

    let new_file_name = "/home/isak102/temptest";
    let new_file = File::create(new_file_name)?; // TODO: generate temp file
    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_writer(new_file);

    for (i, row) in reader.records().enumerate() {
        let record = row?;

        let player_name: String = record[0].to_string();
        let team_short_name: TeamShort = record[1].to_string();
        let team_full_name: TeamFull = record[2].to_string();
        let summoner_name: SummonerName = record[3].to_string();
        let summoner_id: SummonerID = record[4].to_string();

        // TODO: improve this logic below
        if i == 0 {
            writer.write_record(&[
                player_name,
                team_short_name,
                team_full_name,
                summoner_name,
                summoner_id,
            ])?;
            continue;
        }

        // TODO: update summoner name if summoner id exists
        if summoner_id.is_empty() {
            let new_summoner_id = get_summoner_id(&summoner_name).await?;
            writer.write_record(&[
                player_name,
                team_short_name,
                team_full_name,
                summoner_name.clone(), // TODO: fix
                new_summoner_id,
            ])?;
        } else {
            writer.write_record(&[
                player_name,
                team_short_name,
                team_full_name,
                summoner_name,
                summoner_id,
            ])?;
        }
    }

    std::fs::rename(new_file_name, PRO_FILE).expect("Updating data file failed while copying");

    Ok(())
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
    summoner_id: Option<String>,
    game_found: bool,
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
