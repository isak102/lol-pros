use crate::api::RIOT_API;

use super::{RankedStats, SummonerID};
use riven::{
    consts::Tier,
    models::league_v4::{LeagueItem, LeagueList},
    RiotApiError,
};
use std::collections::HashMap;
use tokio::join;

#[derive(Debug)]
pub struct TopLeagues {
    pub players: HashMap<SummonerID, (LeagueItem, Tier)>,
}
impl TopLeagues {
    async fn get_leagues() -> Result<Vec<LeagueList>, RiotApiError> {
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

        Ok(vec![master?, grandmaster?, challenger?])
    }

    pub async fn get() -> Result<Self, RiotApiError> {
        eprintln!("Getting top leagues...");
        let leagues = Self::get_leagues().await?;

        let mut players: HashMap<SummonerID, (LeagueItem, Tier)> = HashMap::with_capacity(5000);

        for league in leagues {
            for entry in league.entries {
                players.insert(entry.summoner_id.clone(), (entry, league.tier));
            }
        }

        eprintln!("Done.");
        Ok(Self { players })
    }

    pub fn get_rank(&self, summoner_id: &str) -> Option<RankedStats> {
        match &self.players.get(summoner_id) {
            Some(&(ref league_item, ref tier)) => Some(RankedStats {
                tier: tier.clone(),
                ranked_data: league_item.clone(),
            }),
            None => None,
        }
    }

    pub fn get_lp(&self, summoner_id: &str) -> Option<i32> {
        self.get_rank(summoner_id)
            .map(|rank| rank.ranked_data.league_points)
    }
}
