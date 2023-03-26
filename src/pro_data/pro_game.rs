use riven::consts::Team;
use std::panic;

use super::*;

#[derive(Debug, Clone)]
pub struct ProGame {
    pub(super) game_info: CurrentGameInfo,
    pub(super) pro_players: Vec<Rc<Pro>>,
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

        // FIXME: add start time, how long the game has gone on for, queue type

        for i in 0..5 {
            let blue_player: &CurrentGameParticipant = blue_team.index(i);
            let red_player: &CurrentGameParticipant = red_team.index(i);

            let extract_info = |player: &CurrentGameParticipant| {
                let pro = self.get_pro(&player.summoner_name);
                let is_pro = pro.is_some();
                let mut pro_name = String::new();

                if is_pro {
                    let pro = pro.unwrap();
                    write!(pro_name, "{} {}", pro.team.short_name, pro.player_name).unwrap();
                }

                (is_pro, pro_name)
            };

            let (blue_is_pro, blue_pro_name) = extract_info(blue_player);
            let (red_is_pro, red_pro_name) = extract_info(red_player);

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

        let (blue_bans, red_bans) = banned_champions_to_string(&self.game_info.banned_champions);
        write!(output, "\n\n\nBlue bans: {blue_bans}\nRed bans: {red_bans}",).unwrap();

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
