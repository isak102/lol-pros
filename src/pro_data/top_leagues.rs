use crate::api::RIOT_API;

use super::{Rank, SummonerID};
use riven::{
    consts::Tier,
    models::league_v4::{LeagueItem, LeagueList},
};
use std::collections::HashMap;
use std::process::exit;
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

        // FIXME: fix these unwraps
        vec![master.unwrap(), grandmaster.unwrap(), challenger.unwrap()]
    }

    pub async fn get() -> Self {
        eprintln!("Getting top leagues...");
        let leagues = Self::get_leagues().await;
        let mut players: HashMap<SummonerID, (LeagueItem, Tier)> = HashMap::new();
        for league_list in leagues {
            for entry in league_list.entries {
                players.insert(entry.summoner_id.clone(), (entry, league_list.tier));
            }
        }
        // dbg!(&players);

        eprintln!("Done.");
        Self { players }
    }

    pub(super) fn get_rank(&self, summoner_id: &str) -> Option<Rank> {
        match &self.players.get(summoner_id) {
            Some(&(ref league_item, ref tier)) => Some(Rank {
                tier: tier.clone(),
                ranked_stats: league_item.clone(),
            }),
            None => None,
        }
    }

    pub fn get_lp(&self, summoner_id: &str) -> Option<i32> {
        self.get_rank(summoner_id)
            .map(|rank| rank.ranked_stats.league_points)
    }
}
