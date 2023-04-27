use super::*;
use chrono::{DateTime, Local, TimeZone};
use riven::consts::Team;
use std::panic;
use std::str;
use strip_ansi_escapes;
use yansi::{Color, Paint};

#[derive(Debug, Clone)]
pub struct ProGame {
    pub(super) game_info: CurrentGameInfo,
    pub(super) pro_players: Vec<Rc<Pro>>,
}

// TODO: move this to ui::raw
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

        // FIXME: add queue type
        writeln!(
            output,
            "Game start time: {}\n", // TODO: print how long ago the start time was
            start_time_to_string(self.game_info.game_start_time)
        )
        .expect("Writing to this buffer should never fail");

        for i in 0..5 {
            let blue_participant: &CurrentGameParticipant = blue_team.index(i);
            let red_participant: &CurrentGameParticipant = red_team.index(i);

            let extract_info = |player: &CurrentGameParticipant| {
                let pro = self.get_pro(&player.summoner_id);
                let is_pro = pro.is_some();
                let mut pro_name = String::new();

                if is_pro {
                    let pro = pro.expect("This should never be None");
                    write!(pro_name, "{} {}", pro.team.short_name, pro.player_name)
                        .expect("Writing to this buffer should never fail");
                }

                (is_pro, pro_name)
            };

            let (blue_is_pro, blue_pro_name) = extract_info(blue_participant);
            let (red_is_pro, red_pro_name) = extract_info(red_participant);

            let blue_player = Paint::wrapping(participant_to_string(
                blue_participant,
                (blue_is_pro, &blue_pro_name),
            ))
            .fg(Color::Blue)
            .to_string();

            let red_player = Paint::wrapping(participant_to_string(
                red_participant,
                (red_is_pro, &red_pro_name),
            ))
            .fg(Color::Red)
            .to_string();

            write!(
                output,
                "{0: <width$} {1}",
                blue_player,
                red_player,
                width = 40 + ansicode_length(&blue_player)
            )?;

            /* dont append newline if we are on the last line */
            if i != 4 {
                output.push('\n');
            }
        }

        let (blue_bans, red_bans) = banned_champions_to_string(&self.game_info.banned_champions);
        write!(
            output,
            "\n\nBlue bans: {}\nRed bans: {}",
            Color::Blue.paint(blue_bans),
            Color::Red.paint(red_bans)
        )
        .expect("Writing to this buffer should never fail");

        write!(f, "{}", output)?;
        Ok(())
    }
}

fn ansicode_length(str: &str) -> usize {
    let stripped_string = str::from_utf8(&strip_ansi_escapes::strip(str).unwrap())
        .unwrap()
        .to_string();

    str.len() - stripped_string.len()
}

// FIXME: game time is very inaccurate, use game_start_time to calculate instead
fn _game_length_to_string(game_length: i64) -> String {
    let minutes = game_length / 60;
    let seconds = game_length % 60;

    format!("{:02}:{:02}", minutes, seconds)
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

impl ProGame {
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
    pub fn get_teams(&self) -> (Vec<&CurrentGameParticipant>, Vec<&CurrentGameParticipant>) {
        let is_red = |p: &&CurrentGameParticipant| p.team_id == Team::RED;

        let (blue, red): (Vec<&CurrentGameParticipant>, Vec<&CurrentGameParticipant>) =
            self.game_info.participants.iter().partition(is_red);

        assert_eq!(blue.len(), 5);
        assert_eq!(red.len(), 5);
        (blue, red)
    }
}
