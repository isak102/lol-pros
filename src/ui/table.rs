use crate::pro_data::ProGame;

use enum_iterator::Sequence;
use lazy_static::lazy_static;
use prettytable::format::{self, Alignment};
use prettytable::{self, color, row, Attr, Cell, Row, Table};
use riven::consts::Team;
use riven::models::spectator_v4::CurrentGameParticipant;
// use termsize;

struct TableData {
    rows: Vec<Vec<CellData>>,
}

impl TableData {
    fn new(pro_game: &ProGame) -> TableData {
        let mut cells: Vec<Vec<CellData>> = Vec::new();
        let (blue_team, red_team) = pro_game.get_teams();

        for (blue_participant, red_participant) in red_team.iter().zip(blue_team.iter()) {
            let f = |participant: &CurrentGameParticipant| {
                let mut player = Vec::new();
                let summoner_name = &participant.summoner_name;
                let pro_name = pro_game.get_pro(&summoner_name);
                let champion_name = participant.champion_id.name().unwrap();

                player.push(CellData {
                    team: participant.team_id,
                    column: Column::ChampionName,
                    raw_string: champion_name.to_string(),
                });
                player.push(CellData {
                    team: participant.team_id,
                    column: Column::SummonerName,
                    raw_string: summoner_name.clone().trim_end().to_string(),
                });
                player.push(CellData {
                    team: participant.team_id,
                    column: Column::ProName,
                    raw_string: match pro_name {
                        Some(pro) => pro.to_string(),
                        None => "".to_string(),
                    },
                });
                player
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

    fn print(&self) {
        let column_lengths = self.get_column_lengths();
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        table.set_titles(row![
            // TODO: use column enum
            "Pro", "Summoner", "Champion", "Champion", "Summoner", "Pro",
        ]);

        assert_eq!(self.rows.len(), 5);
        for row in &self.rows {
            let cells = {
                let mut v = Vec::new();
                for (i, cell) in row.iter().enumerate() {
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
enum Column {
    ChampionName,
    SummonerName,
    ProName,
}

pub fn print(pro_game: &ProGame) -> Result<(), ()> {
    let width = termsize::get().map(|size| size.cols);

    let separator = "—".repeat(width.unwrap_or(120) as usize);

    println!("{separator}");
    TableData::new(pro_game).print();
    println!("{separator}");

    Ok(())
}
