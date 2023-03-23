use csv::{ReaderBuilder, WriterBuilder};
use riven::consts::PlatformRoute;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::{collections::HashMap, io::Result};

use std::rc::Rc;

use riven::models::spectator_v4::*;
use riven::RiotApi;

const PRO_FILE: &str = "/home/isak102/.local/share/pros.csv";
const API_KEY: &str = std::env!("RGAPI_KEY");

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
        let riot_api = RiotApi::new(API_KEY);

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

        // TODO: find way to remove this line
        let participants: Vec<CurrentGameParticipant> = game_info.participants.clone();

        let game = ProGame {
            game_info,
            pro_players: self.find_pros_in_game(participants),
        };

        /* TODO: improve this, ugly af */
        let game_rc = Rc::new(game);
        let game_clone = Rc::clone(&game_rc);
        let game_clone_2 = Rc::clone(&game_rc);

        self.games.push(game_rc);

        self.pros_in_game
            .insert(pro.summoner_name.clone(), game_clone); // TODO: remove .clone()

        Ok(Some(game_clone_2))
    }

    fn find_pros_in_game(&self, summoners: Vec<CurrentGameParticipant>) -> Vec<Rc<Pro>> {
        let mut pros_in_this_game: Vec<Rc<Pro>> = Vec::new();
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

// TODO: implement sync_summoner_names()

// FIXME: rename this function to sync_summoner_ids()
pub async fn sync_data() -> Result<()> {
    let old_file = File::open(PRO_FILE)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(old_file);

    let new_file_name = "/home/isak102/.cache/lolmsi043905-923j39";
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
