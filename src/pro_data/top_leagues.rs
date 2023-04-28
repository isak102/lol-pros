use crate::api::RIOT_API;

use super::SummonerID;
use riven::{
    consts::Tier,
    models::league_v4::{LeagueItem, LeagueList},
};
use std::collections::HashMap;
use tokio::join;

#[derive(Debug)]
pub struct TopLeagues {
    pub players: HashMap<SummonerID, (LeagueItem, Tier)>,
}
impl TopLeagues {
    async fn get_leagues() -> Vec<LeagueList> {
        let (master, grandmaster, challenger) = join!(
            RIOT_API.league_v4().get_master_league(
                riven::consts::PlatformRoute::EUW1,
                riven::consts::QueueType::RANKED_SOLO_5x5
            ),
            RIOT_API.league_v4().get_grandmaster_league(
                riven::consts::PlatformRoute::EUW1,
                riven::consts::QueueType::RANKED_SOLO_5x5
            ),
            RIOT_API.league_v4().get_challenger_league(
                riven::consts::PlatformRoute::EUW1,
                riven::consts::QueueType::RANKED_SOLO_5x5
            ),
        );

        vec![master.unwrap(), grandmaster.unwrap(), challenger.unwrap()]
    }

    pub async fn get() -> Self {
        let leagues = Self::get_leagues().await;
        let mut players: HashMap<SummonerID, (LeagueItem, Tier)> = HashMap::new();
        for league_list in leagues {
            for entry in league_list.entries {
                players.insert(entry.summoner_id.clone(), (entry, league_list.tier));
            }
        }

        Self { players }
    }
}
