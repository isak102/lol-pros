use crate::pro_data::{Pro, ProGame};

use prettytable::format::Alignment;
use prettytable::{self, color, format, row, Attr, Cell, Row};
use riven::consts::Champion;
use riven::consts::Team;
use riven::models::spectator_v4::CurrentGameParticipant;

/// Creates a cell where the text is aligned to `alignment` and the cell size in characters is at
/// least `min_length` long
fn create_cell(string: &str, alignment: Alignment, min_length: usize) -> Cell {
    if string.len() > min_length {
        // TODO: maybe make it shorter?
        return Cell::new(string);
    }

    let whitespace_to_add = min_length - string.len();

    let mut s = String::new();
    match alignment {
        Alignment::LEFT => {
            s.push_str(string);
            s.push_str(" ".repeat(whitespace_to_add).as_str());
            let mut cell = Cell::new(s.as_str());
            cell.align(Alignment::LEFT);
            cell
        }
        Alignment::RIGHT => {
            s.push_str(" ".repeat(whitespace_to_add).as_str());
            s.push_str(string);
            let mut cell = Cell::new(s.as_str());
            cell.align(Alignment::RIGHT);
            cell
        }
        Alignment::CENTER => panic!("Center alignment not supported"),
    }
}

enum CellType {
    Champion(usize),
    SummonerName(usize),
    ProName(usize),
}

fn get_largest_cell_length(pro_game: &ProGame, cell_type: CellType) -> usize {
    let mut largest_length: usize = 0;
    let pros = pro_game.get_participants();

    match cell_type {
        // TODO: clean this up, can be alot better
        CellType::Champion(_) => {
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
        CellType::SummonerName(_) => {
            for participant in pros {
                let val = participant.summoner_name.len();
                if val > largest_length {
                    largest_length = val
                }
            }
            return largest_length;
        }
        CellType::ProName(_) => {
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
    cell_types: &[CellType; 3],
) -> Vec<Cell> {
    let mut chunk = Vec::new();
    let alignment = match team {
        Team::BLUE => Alignment::RIGHT,
        Team::RED => Alignment::LEFT,
        Team::OTHER => panic!("Team can't be OTHER"),
    };
    for cell_type in cell_types {
        match cell_type {
            CellType::Champion(min_length) => {
                let champion_name = player
                    .champion_id
                    .name()
                    .expect("Champion should have a name");
                let cell = create_cell(champion_name, alignment, *min_length);
                chunk.push(cell);
            }
            CellType::SummonerName(min_length) => {
                let cell = create_cell(&player.summoner_name, alignment, *min_length);
                chunk.push(cell);
            }
            CellType::ProName(min_length) => {
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
        CellType::Champion(get_largest_cell_length(
            pro_game,
            CellType::Champion(MIN_CELL_LENGTH),
        )),
        CellType::SummonerName(get_largest_cell_length(
            pro_game,
            CellType::SummonerName(MIN_CELL_LENGTH),
        )),
        CellType::ProName(get_largest_cell_length(
            pro_game,
            CellType::ProName(MIN_CELL_LENGTH),
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
