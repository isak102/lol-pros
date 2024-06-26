use riven::consts::{PlatformRoute, Tier};
use riven::models::league_v4::LeagueItem;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Write};

use std::rc::Rc;

use riven::models::spectator_v4::*;
use riven::RiotApiError;

pub use self::pro_game::*;
pub use self::top_leagues::*;
use super::Config;
use crate::api::RIOT_API;

pub mod io;
mod pro_game;
mod top_leagues;

pub type SummonerID = String;
pub type SummonerName = String;

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
    _full_name: String,
}

pub type RankedData = LeagueItem;
#[derive(Debug, Clone)]
pub struct RankedStats {
    pub tier: Tier,
    pub ranked_data: RankedData,
}

#[derive(Debug)]
pub struct ProData {
    top_leagues: TopLeagues,
    pros: HashMap<SummonerID, Rc<Pro>>,
    games: Vec<Rc<ProGame>>,
    pros_in_game: HashMap<SummonerID, Rc<ProGame>>,
}

impl RankedStats {
    pub fn winrate(&self) -> f32 {
        (self.ranked_data.wins as f32 / (self.ranked_data.wins + self.ranked_data.losses) as f32)
            * 100 as f32
    }

    pub fn games_played(&self) -> u32 {
        self.ranked_data.wins as u32 + self.ranked_data.losses as u32
    }
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
            _full_name: full_name,
        }
    }
}

impl ProData {
    pub async fn load(config: &Config) -> Result<ProData, Box<dyn Error>> {
        let pros = io::load_pros(config).await?;
        Ok(ProData {
            top_leagues: TopLeagues::get().await?,
            pros: pros,
            games: Vec::new(),
            pros_in_game: HashMap::new(),
        })
    }

    /// Gets the LP for a summoner (in master and above)
    /// # Parameter
    /// `summoner_id` - the summoner ID of the summoner
    /// # Returns
    /// `RiotApiError` if getting the league failed
    /// `Ok(Some(lp))` if summoner is above master
    /// `Ok(None)` if summoner isn't ranked or isn't above master
    pub fn get_lp(&self, summoner_id: &str) -> Option<i32> {
        self.top_leagues.get_lp(summoner_id)
    }

    pub fn ranked_stats(&self, summoner_id: &str) -> Option<RankedStats> {
        self.top_leagues.get_rank(summoner_id)
    }

    pub fn pro_leaderboard(&self) -> Vec<(Rc<Pro>, RankedStats)> {
        let mut result = Vec::new();
        for pro in self.get_pros() {
            let rank = match self.ranked_stats(&pro.summoner_id.as_ref().unwrap()) {
                Some(r) => r,
                None => continue,
            };
            result.push((pro, rank));
        }

        result.sort_by_key(|&(_, ref r)| r.ranked_data.league_points);
        result.reverse();
        result
    }

    // TODO: find way to return Vec<&Pro>
    pub fn get_pros(&self) -> Vec<Rc<Pro>> {
        let mut result = Vec::new();
        for (_, val) in &self.pros {
            result.push(Rc::clone(&val));
        }
        result
    }

    pub fn is_in_game(&self, pro: &Pro) -> bool {
        let summoner_id = pro.summoner_id.clone().unwrap_or("".to_string());
        self.pros_in_game.contains_key(summoner_id.as_str())
    }

    pub async fn fetch_game(
        &mut self,
        pro: &Pro,
    ) -> std::result::Result<Option<Rc<ProGame>>, RiotApiError> {
        let summoner_id: &SummonerID = match &pro.summoner_id {
            Some(id) => id,
            None => {
                eprintln!("{} has no summoner ID, can't find game. Quitting.", pro);
                std::process::exit(1);
            }
        };

        /* If this pro already is in a found game then we return that game instantly */
        if let Some(game) = self
            .pros_in_game
            .get(pro.summoner_id.clone().unwrap().as_str())
        {
            dbg!("Found pro in game");
            return Ok(Some(Rc::clone(&game)));
        }

        let game_info = match RIOT_API
            .spectator_v4()
            .get_current_game_info_by_summoner(PlatformRoute::EUW1, summoner_id)
            .await?
        {
            Some(g) => g,
            None => return Ok(None),
        };

        /* Return none if game is not ranked game */
        if let Some(q) = game_info.game_queue_config_id {
            if q != riven::consts::Queue::SUMMONERS_RIFT_5V5_RANKED_SOLO {
                return Ok(None);
            }
        }

        let pro_players = self.find_pros_in_game(&game_info);
        let mut players = Vec::new();
        for participant in &game_info.participants {
            let p = Player::new(participant.summoner_id.as_str(), participant.clone(), &self)
                .expect("Couldnt create Player");
            players.push(p);
        }

        let game = Rc::new(ProGame {
            players,
            game_info,
            pro_players: pro_players,
        });

        /* Insert each pro player in this game into the hashmap of pro_players that are in game. */
        for pro_player in &game.pro_players {
            self.pros_in_game
                .insert(pro_player.summoner_id.clone().unwrap(), Rc::clone(&game));
        }

        let game_clone = Rc::clone(&game);
        self.games.push(game);

        Ok(Some(game_clone))
    }

    fn find_pros_in_game(&self, game_info: &CurrentGameInfo) -> Vec<Rc<Pro>> {
        let mut pros_in_this_game: Vec<Rc<Pro>> = Vec::new();
        let summoners: &Vec<CurrentGameParticipant> = &game_info.participants;
        for summoner in summoners {
            let summoner_id = &summoner.summoner_id;

            match self.pros.get(summoner_id) {
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

    pub fn pros_count(&self) -> usize {
        self.pros.len()
    }
}

impl Display for RankedStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let tier_str = match self.tier {
            Tier::CHALLENGER => "C1",
            Tier::GRANDMASTER => "GM",
            Tier::MASTER => "M",
            _ => panic!("Rank should never be below master"),
        };

        write!(
            f,
            "{} {}LP {}gp {:.1}%",
            tier_str,
            self.ranked_data.league_points,
            self.games_played(),
            self.winrate()
        )
    }
}
