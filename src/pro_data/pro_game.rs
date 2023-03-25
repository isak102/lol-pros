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

        // TODO: extract into function
        write!(
            output,
            "\n\nBanned champions: {:?}",
            &self.game_info.banned_champions
        )
        .unwrap();

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
