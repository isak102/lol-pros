use crate::pro_data::{Pro, ProGame};

use enum_iterator::Sequence;
use lazy_static::lazy_static;
use prettytable::format::Alignment;
use prettytable::{self, color, format, row, Attr, Cell, Row, Table};
use riven::consts::Champion;
use riven::consts::Team;
use riven::models::spectator_v4::CurrentGameParticipant;

struct TableData<'a> {
    cells: Vec<Vec<CellData<'a>>>,
}

impl<'a> TableData<'a> {
    fn new(pro_game: &'a ProGame) -> Self {
        let mut cells: Vec<Vec<CellData<'a>>> = Vec::new();
        let (blue_team, red_team) = pro_game.get_teams();

        for (blue_participant, red_participant) in blue_team.iter().zip(red_team.iter()) {
            let f = |participant: &CurrentGameParticipant| {
                let mut player = Vec::new();
                let summoner_name = participant.summoner_name.clone();
                let pro_name = pro_game.get_pro(&summoner_name);
                let champion_name = participant.champion_id.name().unwrap();

                player.push(CellData {
                    team: participant.team_id,
                    column: Column::ProName,
                    raw_str: match pro_name {
                        Some(pro) => pro.to_string().as_str(),
                        None => "None",
                    },
                });
                player.push(CellData {
                    team: participant.team_id,
                    column: Column::SummonerName,
                    raw_str: summoner_name.as_str(),
                });
                player.push(CellData {
                    team: participant.team_id,
                    column: Column::ChampionName,
                    raw_str: champion_name,
                });
                player
            };

            let (mut blue_player, mut red_player) = (f(blue_participant), f(red_participant));

            blue_player.reverse();
            blue_player.append(&mut red_player);
            cells.push(blue_player);
        }

        Self { cells }
    }

    fn get_column_lengths(&self) -> Vec<usize> {
        let mut column_lengths = Vec::new();

        for column in ALL_COLUMNS.iter() {
            let mut max_length = 0;
            for row in &self.cells {
                let cell = row
                    .iter()
                    .find(|cell| cell.column == *column)
                    .expect("Cell should exist");
                let length = cell.get_str_length();
                if length > max_length {
                    max_length = length;
                }
            }
            column_lengths.push(max_length);
        }
        assert_eq!(column_lengths.len(), ALL_COLUMNS.len());
        column_lengths
    }

    fn print(&self) {
        let column_lengths = self.get_column_lengths();
        let mut table = Table::new();

        for row in self.cells.iter() {
            let row = {
                let mut v = Vec::new();
                for (i, cell) in row.iter().enumerate() {
                    v.push(cell.make_cell(column_lengths[i]))
                }
                v
            };

            table.add_row(Row::new(row));
        }
        table.printstd();
    }
}

struct CellData<'a> {
    team: Team,
    column: Column,
    raw_str: &'a str,
}

impl<'a> CellData<'a> {
    fn make_cell(&self, length: usize) -> Cell {
        let string = self.raw_str;
        assert!(length >= string.len());

        let whitespace_to_add = length - string.len();

        let mut s = String::new();
        match self.team {
            Team::BLUE => {
                s.push_str(" ".repeat(whitespace_to_add).as_str());
                s.push_str(string);
                let mut cell = Cell::new(s.as_str());
                cell.align(Alignment::RIGHT);
                cell.with_style(Attr::ForegroundColor(color::RED));
                cell
            }
            Team::RED => {
                s.push_str(string);
                s.push_str(" ".repeat(whitespace_to_add).as_str());
                let mut cell = Cell::new(s.as_str());
                cell.align(Alignment::LEFT);
                cell.with_style(Attr::ForegroundColor(color::BLUE));
                cell
            }
            Team::OTHER => panic!("Summoner should be BLUE or RED team"),
        }
    }

    fn get_str_length(&self) -> usize {
        self.raw_str.len()
    }
}

lazy_static! {
    static ref ALL_COLUMNS: Vec<Column> = enum_iterator::all::<Column>().collect::<Vec<_>>();
}

#[derive(Sequence, Eq, PartialEq)]
enum Column {
    ChampionName,
    SummonerName,
    ProName,
}

fn get_largest_cell_length(pro_game: &ProGame, cell_type: Column) -> usize {
    let mut largest_length: usize = 0;
    let pros = pro_game.get_participants();

    match cell_type {
        // TODO: clean this up, can be alot better
        Column::ChampionName(_) => {
            for participant in pros {
                let val = participant
                    .champion_id
                    .name()
                    .expect("Champion should have a name")
                    .len();
                if val > largest_length {
                    largest_length = val
                }
            }
            return largest_length;
        }
        Column::SummonerName(_) => {
            for participant in pros {
                let val = participant.summoner_name.len();
                if val > largest_length {
                    largest_length = val
                }
            }
            return largest_length;
        }
        Column::ProName(_) => {
            let pros = pro_game.get_pro_players();
            for pro in pros {
                let val = pro.to_string().len();
                if val > largest_length {
                    largest_length = val
                }
            }
            return largest_length;
        }
    }
}

fn create_player_chunk(
    player: &CurrentGameParticipant,
    pro: Option<&Pro>,
    team: Team,
    cell_types: &[Column; 3],
) -> Vec<Cell> {
    let mut chunk = Vec::new();
    let alignment = match team {
        Team::BLUE => Alignment::RIGHT,
        Team::RED => Alignment::LEFT,
        Team::OTHER => panic!("Team can't be OTHER"),
    };
    for cell_type in cell_types {
        match cell_type {
            Column::ChampionName(min_length) => {
                let champion_name = player
                    .champion_id
                    .name()
                    .expect("Champion should have a name");
                let cell = create_cell(champion_name, alignment, *min_length);
                chunk.push(cell);
            }
            Column::SummonerName(min_length) => {
                let cell = create_cell(&player.summoner_name, alignment, *min_length);
                chunk.push(cell);
            }
            Column::ProName(min_length) => {
                // TODO: fix this
                let s = match pro {
                    Some(p) => p.to_string(),
                    None => "".to_string(),
                };
                let cell = create_cell(s.as_str(), alignment, *min_length);
                chunk.push(cell);
            }
        }
    }
    if team == Team::BLUE {
        chunk.reverse();
    }

    return chunk;
}

pub fn print(pro_game: &ProGame) -> Result<(), ()> {
    let mut table = prettytable::Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row![
        "Pro", "Summoner", "Champion", "Champion", "Summoner", "Pro",
    ]);

    const MIN_CELL_LENGTH: usize = 10;

    let cell_lenths = [
        // TODO: improve this, it is really ugly
        Column::ChampionName(get_largest_cell_length(
            pro_game,
            Column::ChampionName(MIN_CELL_LENGTH),
        )),
        Column::SummonerName(get_largest_cell_length(
            pro_game,
            Column::SummonerName(MIN_CELL_LENGTH),
        )),
        Column::ProName(get_largest_cell_length(
            pro_game,
            Column::ProName(MIN_CELL_LENGTH),
        )),
    ];

    let (blue_team, red_team) = pro_game.get_teams();
    for (blue_player, red_player) in blue_team.iter().zip(red_team.iter()) {
        let blue_player_chunk = create_player_chunk(
            blue_player,
            pro_game.get_pro(&blue_player.summoner_name),
            Team::BLUE,
            &cell_lenths,
        );
        let red_player_chunk = create_player_chunk(
            red_player,
            pro_game.get_pro(&red_player.summoner_name),
            Team::RED,
            &cell_lenths,
        );

        let mut row = Row::empty();
        for cell in blue_player_chunk {
            row.add_cell(cell.with_style(Attr::ForegroundColor(color::BLUE)));
        }
        for cell in red_player_chunk {
            row.add_cell(cell.with_style(Attr::ForegroundColor(color::RED)));
        }
        table.add_row(row);
    }

    table.printstd();
    Ok(())
}
