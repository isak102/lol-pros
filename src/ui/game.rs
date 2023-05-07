use std::fmt::{self, Display, Formatter};

use crate::pro_data::{Player, ProGame};

use enum_iterator::Sequence;
use lazy_static::lazy_static;
use prettytable::format::{self, Alignment};
use prettytable::{self, color, Attr, Cell, Row, Table};
use riven::consts::Team;

struct TableData {
    rows: Vec<Vec<CellData>>,
}

impl TableData {
    fn new(pro_game: &ProGame) -> TableData {
        let mut cells: Vec<Vec<CellData>> = Vec::new();
        let (blue_team, red_team) = pro_game.teams();

        for (blue_participant, red_participant) in red_team.iter().zip(blue_team.iter()) {
            let f = |player: &Player| {
                let mut player_row = Vec::new();
                let summoner_id = &player.current_game_participant.summoner_id;
                let summoner_name = &player.current_game_participant.summoner_name;
                let pro_name = pro_game.get_pro(&summoner_id);
                let champion_name = player.current_game_participant.champion_id.name().unwrap();

                for column in ALL_COLUMNS.iter().rev() {
                    match column {
                        Column::ProName => {
                            player_row.push(CellData {
                                team: player.current_game_participant.team_id,
                                column: Column::ProName,
                                raw_string: match pro_name {
                                    Some(pro) => pro.to_string(),
                                    None => "".to_string(),
                                },
                            });
                        }
                        Column::RankInfo => {
                            let rank_str = match player.ranked_stats() {
                                Some(rank) => rank.to_string(),
                                None => "-".to_string(),
                            };
                            player_row.push(CellData {
                                team: player.current_game_participant.team_id,
                                column: Column::RankInfo,
                                raw_string: rank_str,
                            });
                        }
                        Column::SummonerName => {
                            player_row.push(CellData {
                                team: player.current_game_participant.team_id,
                                column: Column::SummonerName,
                                raw_string: summoner_name.trim_end().to_string(),
                            });
                        }
                        Column::ChampionName => {
                            player_row.push(CellData {
                                team: player.current_game_participant.team_id,
                                column: Column::ChampionName,
                                raw_string: champion_name.to_string(),
                            });
                        }
                    }
                }

                player_row
            };

            let (mut blue_player, mut red_player) = (f(blue_participant), f(red_participant));

            blue_player.reverse();
            blue_player.append(&mut red_player);
            cells.push(blue_player);
        }

        Self { rows: cells }
    }

    fn get_column_lengths(&self) -> Vec<usize> {
        let mut column_lengths = Vec::new();
        const PADDING: usize = 1;

        for column in ALL_COLUMNS.iter() {
            let max_length = self
                .rows
                .iter()
                .flat_map(|row| row.iter().filter(|cell| cell.column == *column))
                .map(|cell| cell.get_str_length() + PADDING)
                .max()
                .unwrap_or(0);

            column_lengths.push(max_length);
        }
        assert_eq!(column_lengths.len(), ALL_COLUMNS.len());

        column_lengths
    }

    fn get_title_row() -> Row {
        let mut column_strings: Vec<String> = Vec::new();
        let mut result = Vec::new();

        for column in ALL_COLUMNS.iter() {
            column_strings.push(column.to_string());
        }

        let mut f = |s| {
            let mut c = Cell::new(s);
            c.style(Attr::Bold);
            c.align(Alignment::CENTER);
            result.push(c)
        };

        for column_string in column_strings.iter() {
            f(column_string);
        }
        for column_string in column_strings.iter().rev() {
            f(column_string);
        }

        Row::new(result)
    }

    fn print(&self) {
        let column_lengths = self.get_column_lengths();
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(Self::get_title_row());

        assert_eq!(self.rows.len(), 5);
        for row in &self.rows {
            let cells = {
                let mut v = Vec::new();
                for cell in row {
                    v.push(cell.make_cell(column_lengths[cell.clone().column as usize]))
                }
                v
            };

            table.add_row(Row::new(cells));
        }
        table.printstd();
    }
}

#[derive(Debug)]
struct CellData {
    team: Team,
    column: Column,
    raw_string: String,
}

impl CellData {
    fn make_cell(&self, length: usize) -> Cell {
        assert!(length + 1 >= self.raw_string.len());

        let whitespace_to_add = length - self.raw_string.len();

        let mut s = String::new();
        match self.team {
            Team::BLUE => {
                s.push_str(" ".repeat(whitespace_to_add).as_str());
                s.push_str(self.raw_string.as_str());
                let mut cell = Cell::new(s.as_str());
                cell.align(Alignment::RIGHT);
                cell.style(Attr::ForegroundColor(color::BLUE));
                cell
            }
            Team::RED => {
                s.push_str(self.raw_string.as_str());
                s.push_str(" ".repeat(whitespace_to_add).as_str());
                let mut cell = Cell::new(s.as_str());
                cell.align(Alignment::LEFT);
                cell.style(Attr::ForegroundColor(color::RED));
                cell
            }
            Team::OTHER => panic!("Summoner should be BLUE or RED team"),
        }
    }

    fn get_str_length(&self) -> usize {
        self.raw_string.len()
    }
}

lazy_static! {
    static ref ALL_COLUMNS: Vec<Column> = enum_iterator::all::<Column>().collect::<Vec<_>>();
}

#[derive(Sequence, Eq, PartialEq, Debug, Copy, Clone)]
/// Columns ordered from left to right
enum Column {
    ProName,
    RankInfo,
    SummonerName,
    ChampionName,
}

impl Display for Column {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Column::ProName => write!(f, "Pro"),
            Column::RankInfo => write!(f, "Rank"),
            Column::SummonerName => write!(f, "Summoner"),
            Column::ChampionName => write!(f, "Champion"),
        }
    }
}

pub async fn print(pro_game: &ProGame) -> Result<(), ()> {
    let width = termsize::get().map(|size| size.cols);

    let separator = "—".repeat(width.unwrap_or(120) as usize);

    println!("{separator}");
    eprintln!("{}LP", pro_game.average_lp());
    TableData::new(pro_game).print();
    println!("{separator}");

    Ok(())
}
