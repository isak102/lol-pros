use super::*;
use csv::{ReaderBuilder, WriterBuilder};
use std::fs::File;

pub(super) async fn load_pros(config: &Config) -> Result<HashMap<String, Rc<Pro>>> {
    sync_summoner_ids(config).await?; // TODO: maybe remove this

    let file = File::open(&config.pro_file_path)?;
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut pros = HashMap::new();

    for result in reader.records() {
        let record = result?;

        let player_name: PlayerName = record[0].to_string();
        let team_short_name: TeamShort = record[1].to_string();
        let team_full_name: TeamFull = record[2].to_string();
        let summoner_name: SummonerName = record[3].to_string();
        let summoner_id: SummonerID = record[4].to_string();

        let team = Team::new(team_short_name, team_full_name);
        let pro = Pro::new(player_name, team, summoner_name.clone(), summoner_id);

        pros.insert(summoner_name, Rc::new(pro));
    }

    Ok(pros)
}

pub(super) async fn sync_summoner_ids(config: &Config) -> Result<()> {
    let old_file = File::open(&config.pro_file_path)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(old_file);

    let new_file_name = "/home/isak102/.cache/lolmsi043905-923j39";
    let new_file = File::create(new_file_name)?;
    let mut writer = WriterBuilder::new().has_headers(true).from_writer(new_file);

    for row in reader.records() {
        let record = row?;

        let player_name: String = record[0].to_string();
        let team_short_name: TeamShort = record[1].to_string();
        let team_full_name: TeamFull = record[2].to_string();
        let summoner_name: SummonerName = record[3].to_string();

        let summoner_id: SummonerID = record[4].to_string();
        let summoner_id = if summoner_id.is_empty() {
            get_summoner_id(&summoner_name).await?
        } else {
            summoner_id
        };

        writer.write_record(&[
            player_name,
            team_short_name,
            team_full_name,
            summoner_name,
            summoner_id,
        ])?;
    }

    std::fs::rename(new_file_name, &config.pro_file_path)
        .expect("Updating data file failed while copying");

    Ok(())
}

async fn get_summoner_id(summoner_name: &SummonerName) -> Result<SummonerID> {
    let riot_api = RiotApi::new(api_key::API_KEY);

    let summoner = match riot_api
        .summoner_v4()
        .get_by_summoner_name(PlatformRoute::EUW1, summoner_name)
        .await
    {
        Ok(s) => match s {
            Some(summoner) => summoner,
            None => return Err(Error::new(ErrorKind::Other, "Summoner not found")),
        },
        Err(_) => return Err(Error::new(ErrorKind::Other, "Error getting summoner info")),
    };

    Ok(summoner.id)
}
