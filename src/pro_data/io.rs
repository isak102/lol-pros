use super::*;
use crate::api::RIOT_API;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use std::io::{Error as IoError, ErrorKind};
use std::{error::Error, fs::File};

#[derive(serde::Deserialize, serde::Serialize)]
struct Row {
    pro_name: String,
    short_team: String,
    long_team: String,
    summoner_name: String,
    summoner_id: String,
}

lazy_static::lazy_static! {
    pub static ref CSV_HEADER: StringRecord = StringRecord::from(vec![
        "pro_name",
        "short_team",
        "long_team",
        "summoner_name",
        "summoner_id",
    ]);
}

pub(super) async fn load_pros(config: &Config) -> Result<HashMap<String, Rc<Pro>>, Box<dyn Error>> {
    let file = File::open(&config.pro_file_path)?;
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut pros = HashMap::new();

    for record in reader.records() {
        let row: Row = match record {
            Ok(r) => match r.deserialize(Some(&CSV_HEADER)) {
                Ok(r) => r,
                Err(_) => {
                    dbg!("Error reading record");
                    continue;
                }
            },
            Err(e) => {
                eprintln!("Error CSV record {e}, skipping line");
                continue;
            }
        };

        let team = Team::new(row.short_team, row.long_team);
        let pro = Pro::new(
            row.pro_name,
            team,
            row.summoner_name,
            row.summoner_id.clone(),
        );

        pros.insert(row.summoner_id, Rc::new(pro));
    }

    Ok(pros)
}

pub async fn sync_summoner_ids(config: &Config) -> Result<(), Box<dyn Error>> {
    let old_file = File::open(&config.pro_file_path)?;
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(old_file);

    const NEW_FILE_NAME: &str = "/home/isak102/.cache/lolmsi043905-923j39";
    let new_file = File::create(NEW_FILE_NAME)?;
    let mut writer = WriterBuilder::new().has_headers(true).from_writer(new_file);

    for record in reader.records() {
        let row: Row = match record {
            Ok(r) => match r.deserialize(Some(&CSV_HEADER)) {
                Ok(r) => r,
                Err(_) => {
                    dbg!("Error reading record");
                    continue;
                }
            },
            Err(e) => {
                eprintln!("Error CSV record {e}, skipping line");
                continue;
            }
        };

        let summoner_id = if row.summoner_id.is_empty() {
            let s = get_summoner_id(&row.summoner_name).await?;
            eprintln!("Found summoner id for {}: {}", row.pro_name, s);
            s
        } else {
            row.summoner_id
        };

        let new_row = Row {
            pro_name: row.pro_name,
            short_team: row.short_team,
            long_team: row.long_team,
            summoner_name: row.summoner_name,
            summoner_id,
        };

        writer.serialize(new_row)?;
    }

    std::fs::rename(NEW_FILE_NAME, &config.pro_file_path)
        .expect("Updating data file failed while copying");

    Ok(())
}

async fn get_summoner_id(summoner_name: &SummonerName) -> Result<SummonerID, Box<dyn Error>> {
    let summoner = match RIOT_API
        .summoner_v4()
        .get_by_summoner_name(PlatformRoute::EUW1, summoner_name)
        .await?
    {
        Some(summoner) => summoner,
        None => {
            return Err(Box::new(IoError::new(
                ErrorKind::Other,
                format!("Could not find summoner {}", summoner_name),
            )));
        }
    };

    Ok(summoner.id)
}
