use csv::ReaderBuilder;
use std::fs::File;
use std::{collections::HashMap, io::Result};

const PRO_FILE: &str = "/home/isak102/.local/share/pros.csv";

pub type AccountID = String;

pub struct Team {
    pub short_name: String,
    pub full_name: String,
}

impl Team {
    pub fn new(short_name: String, full_name: String) -> Team {
        Team {
            short_name,
            full_name,
        }
    }
}

pub struct Pro {
    pub player_name: String,
    pub team: Team,
    pub summoner_name: String,
    account_id: Option<AccountID>,
}

impl Pro {
    pub fn new(
        player_name: String,
        team: Team,
        summoner_name: String,
        account_id: Option<AccountID>,
    ) -> Pro {
        Pro {
            player_name,
            team,
            summoner_name,
            account_id,
        }
    }
}

pub fn get_pros() -> Result<HashMap<String, Pro>> {
    let file = File::open(PRO_FILE)?;
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut pros: HashMap<String, Pro> = HashMap::new();

    for result in reader.records() {
        let record = result?;

        let player_name: String = record[0].to_string();
        let team_short_name: String = record[1].to_string();
        let team_full_name: String = record[2].to_string();
        let summoner_name: String = record[3].to_string();

        let account_id: Option<AccountID>;
        if record[4].is_empty() {
            account_id = None;
        } else {
            account_id = Some(record[4].to_string());
        }
        
        let team = Team::new(team_short_name, team_full_name);
        let pro = Pro::new(player_name, team, summoner_name.clone(), account_id);

        pros.insert(summoner_name, pro);
    }

    Ok(pros)
}

pub fn update_pros() -> Result<()> {
    Ok(())
}
