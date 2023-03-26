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

        // FIXME: add start time, queue type
        write!(
            output,
            "{}\n\n",
            game_length_to_string(self.game_info.game_length)
        )
        .expect("Writing to this buffer should never fail");

        for i in 0..5 {
            let blue_player: &CurrentGameParticipant = blue_team.index(i);
            let red_player: &CurrentGameParticipant = red_team.index(i);

            let extract_info = |player: &CurrentGameParticipant| {
                let pro = self.get_pro(&player.summoner_name);
                let is_pro = pro.is_some();
                let mut pro_name = String::new();

                if is_pro {
                    let pro = pro.expect("This should never be None");
                    write!(pro_name, "{} {}", pro.team.short_name, pro.player_name)
                        .expect("Writing to this buffer should never fail");
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
        write!(output, "\n\n\nBlue bans: {blue_bans}\nRed bans: {red_bans}",)
            .expect("Writing to this buffer should never fail");

        write!(f, "{}", output)?;
        Ok(())
    }
}

fn game_length_to_string(game_length: i64) -> String {
    let minutes = game_length / 60;
    let seconds = game_length % 60;

    format!("{:02}:{:02}", minutes, seconds)
}

fn participant_to_string(participant: &CurrentGameParticipant, is_pro: (bool, &str)) -> String {
    let mut result = String::new();
    if let (true, pro_name) = is_pro {
        write!(result, "<{}> ", pro_name).expect("Writing to this buffer should never fail");
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
