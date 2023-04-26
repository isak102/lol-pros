use clap::Parser;
use yansi::Paint;

pub struct Config {
    pub pro_file_path: String, // TODO: turn this into a path
    pub sync_summoner_names: bool,
}

#[derive(Parser, Debug)]
struct Args {
    /// Path to the CSV file containing pros
    #[arg(short, long, default_value = "/home/isak102/.local/share/pros.csv")]
    pro_file_path: String,

    /// Disable colors (CLICOLOR=0 takes precedence over this)
    #[arg(short, long)]
    disable_colors: bool,
}

impl Config {
    pub fn parse() -> Result<Config, &'static str> {
        let args = Args::parse();
        let disable_colors;

        // disable color if CLICOLOR is set to 0
        if let Ok(true) = std::env::var("CLICOLOR").map(|v| v == "0") {
            disable_colors = true;
        } else {
            disable_colors = args.disable_colors;
        }
        if disable_colors {
            Paint::disable();
        }

        let sync_summoner_names = false; // TODO: implement support for this
        let pro_file_path = args.pro_file_path;

        Ok(Config {
            pro_file_path,
            sync_summoner_names,
        })
    }
}
