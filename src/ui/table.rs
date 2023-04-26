use crate::pro_data::ProGame;

use prettytable::{self, format, Attr, Cell, Row, row};
use riven::consts::Champion;

pub fn print(pro_game: &ProGame) -> Result<(), ()> {
    let mut table = prettytable::Table::new();

    table.add_row(row![
        "Team",
        "",
    ]);

    table.printstd();

    todo!()
}

fn create_champ_cell(champion_id: &Champion) -> Cell {
    let champion_name = champion_id.name().expect("Champion should have a name");
    let mut cell = Cell::new(champion_name).with_style(Attr::Bold);

    cell
}
