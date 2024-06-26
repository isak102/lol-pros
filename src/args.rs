use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Args {
    /// Path to the CSV file containing pros
    #[arg(short, long, default_value = "/home/isak102/.local/share/pros.csv")]
    pub pro_file_path: String,

    // TODO: find way to disable color for table printing too
    /// Disable colors [doesn't work with tables] (CLICOLOR=0 takes precedence over this option)
    #[arg(short, long)]
    pub disable_colors: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Sync pro data
    #[command(alias = "s")]
    Sync {},

    /// Print pro players leaderboard
    #[command(alias = "l")]
    Leaderboard {},
}
