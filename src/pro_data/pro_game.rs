#![allow(dead_code)]

use super::*;
use chrono::{DateTime, Local, TimeZone};
use riven::consts::Team;
use std::panic;
use std::str;
use strip_ansi_escapes;
use tokio::task;
use yansi::{Paint};

#[derive(Debug, Clone)]
pub struct ProGame {
    pub(super) game_info: CurrentGameInfo,
    pub(super) pro_players: Vec<Rc<Pro>>,
}

impl ProGame {
    /// Get pro by summoner_id
    /// # Parameters
    /// `summoner_id` - Summoner ID of the pro
    /// # Returns
    /// - `Some(p)` if pro was found
    /// - `None` if pro was not found
    pub fn get_pro(&self, summoner_id: &str) -> Option<&Pro> {
        let p = self
            .pro_players
            .iter()
            .find(|pro| pro.summoner_id.as_deref() == Some(summoner_id));

        match p {
            Some(p) => Some(p.as_ref()),
            None => None,
        }
    }

    /// Get the teams in the game
    /// # Returns
    /// A tuple of vectors with references to each player in the team. Both vectors will have the
    /// size 5
    pub fn teams(&self) -> (Vec<&CurrentGameParticipant>, Vec<&CurrentGameParticipant>) {
        let is_red = |p: &&CurrentGameParticipant| p.team_id == Team::RED;

        let (blue, red): (Vec<&CurrentGameParticipant>, Vec<&CurrentGameParticipant>) =
            self.game_info.participants.iter().partition(is_red);

        assert_eq!(blue.len(), 5);
        assert_eq!(red.len(), 5);
        (blue, red)
    }

    pub async fn average_lp(&self) -> usize {
        let mut total_lp = 0;
        let mut tasks = vec![];

        for participant in &self.game_info.participants {
            let summoner_id = participant.summoner_id.clone();
            let handle = task::spawn(get_lp(summoner_id));
            tasks.push(handle);
        }

        for handle in tasks {
            total_lp += handle.await.unwrap().unwrap().unwrap();
        }

        total_lp / &self.game_info.participants.len()
    }
}

// FIXME: Move the functions in this file somewhere else

// FIXME: game time is very inaccurate, use game_start_time to calculate instead
fn game_length_to_string(game_length: i64) -> String {
    let minutes = game_length / 60;
    let seconds = game_length % 60;

    format!("{:02}:{:02}", minutes, seconds)
}

fn ansicode_length(str: &str) -> usize {
    let stripped_string = str::from_utf8(&strip_ansi_escapes::strip(str).unwrap())
        .unwrap()
        .to_string();

    str.len() - stripped_string.len()
}

fn start_time_to_string(start_time: i64) -> String {
    let local = epoch_ms_to_local_time(start_time);
    local.format("%X").to_string()
}

fn epoch_ms_to_local_time(epoch_ms: i64) -> DateTime<Local> {
    let tz = Local::now().timezone(); // Get local timezone
    let dt = tz.timestamp_millis_opt(epoch_ms).unwrap(); // Convert epoch milliseconds to DateTime
    dt.with_timezone(&tz) // Convert DateTime to local timezone
}

fn participant_to_string(participant: &CurrentGameParticipant, is_pro: (bool, &str)) -> String {
    let mut result = String::new();

    if let (true, pro_name) = is_pro {
        let pro_name_wrapped = format!("<{}>", pro_name);
        let pro_name_colored = Paint::yellow(pro_name_wrapped).bold();

        write!(result, "{} ", pro_name_colored).expect("Writing to this buffer should never fail");
    }

    write!(
        result,
        "{} [{}]",
        participant
            .champion_id
            .name()
            .expect("Champion should have a name"),
        participant.summoner_name,
    )
    .expect("Writing to this buffer should never fail");

    result
}

fn banned_champions_to_string(banned_champions: &Vec<BannedChampion>) -> (String, String) {
    let mut blue_string = String::new();
    let mut red_string = String::new();

    let _: Vec<&BannedChampion> = banned_champions
        .iter()
        .map(|champ| {
            let push_champ_string = |s: &mut String| {
                let banned_champ_str = match champ.champion_id.name() {
                    Some(champ) => champ,
                    None => "None",
                };

                s.push_str(format!("{banned_champ_str}, ").as_str());
            };

            match champ.team_id {
                Team::BLUE => {
                    push_champ_string(&mut blue_string);
                }
                Team::RED => {
                    push_champ_string(&mut red_string);
                }
                _ => panic!("Champion was not banned by either BLUE or RED team"),
            }

            champ
        })
        .collect();

    let suffix = ", ";
    let remove_suffix = |s: &mut String| {
        if let Some(stripped) = s.strip_suffix(suffix) {
            stripped.to_string()
        } else {
            panic!("Strip should not fail here");
        }
    };

    (
        remove_suffix(&mut blue_string),
        remove_suffix(&mut red_string),
    )
}
