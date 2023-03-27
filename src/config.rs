use yansi::Paint;

pub struct Config {
    pub pro_file_path: String,
    pub sync_summoner_names: bool,
}

impl Config {
    pub fn parse(args: &[String]) -> Result<Config, &'static str> {
        
        // disable color if CLICOLOR is set to 0
        if let Ok(true) = std::env::var("CLICOLOR").map(|v| v == "0") {
            Paint::disable();
        }

        let sync_summoner_names = false; // TODO: implement support for this

        let pro_file_path = if args.len() > 1 {
            args[1].clone()
        } else {
            String::from("/home/isak102/.local/share/pros.csv")
        };

        Ok(Config {
            pro_file_path,
            sync_summoner_names,
        })
    }
}
