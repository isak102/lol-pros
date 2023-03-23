pub mod sync_data {
    use super::super::*;
    use csv::{ReaderBuilder, WriterBuilder};
    use std::fs::File;

    // FIXME: rename this function to sync_summoner_ids()
    pub async fn sync_summoner_ids() -> Result<()> {
        let old_file = File::open(PRO_FILE)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(old_file);

        let new_file_name = "/home/isak102/.cache/lolmsi043905-923j39";
        let new_file = File::create(new_file_name)?; // TODO: generate temp file
        let mut writer = WriterBuilder::new()
            .has_headers(false)
            .from_writer(new_file);

        for (i, row) in reader.records().enumerate() {
            let record = row?;

            let player_name: String = record[0].to_string();
            let team_short_name: TeamShort = record[1].to_string();
            let team_full_name: TeamFull = record[2].to_string();
            let summoner_name: SummonerName = record[3].to_string();
            let summoner_id: SummonerID = record[4].to_string();

            // TODO: improve this logic below
            if i == 0 {
                writer.write_record(&[
                    player_name,
                    team_short_name,
                    team_full_name,
                    summoner_name,
                    summoner_id,
                ])?;
                continue;
            }

            // TODO: update summoner name if summoner id exists
            if summoner_id.is_empty() {
                let new_summoner_id = get_summoner_id(&summoner_name).await?;
                writer.write_record(&[
                    player_name,
                    team_short_name,
                    team_full_name,
                    summoner_name.clone(), // TODO: fix
                    new_summoner_id,
                ])?;
            } else {
                writer.write_record(&[
                    player_name,
                    team_short_name,
                    team_full_name,
                    summoner_name,
                    summoner_id,
                ])?;
            }
        }

        std::fs::rename(new_file_name, PRO_FILE).expect("Updating data file failed while copying");

        Ok(())
    }
    
    async fn get_summoner_id(summoner_name: &SummonerName) -> Result<SummonerID> {
        let riot_api = RiotApi::new(API_KEY);

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
}
