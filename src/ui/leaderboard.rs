use crate::pro_data::ProData;
use prettytable::{format, row, Table};

pub fn print(pro_data: &ProData) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_CLEAN);
    let leaderboard = pro_data.pro_leaderboard();
    for (i, (pro, rank)) in leaderboard.iter().enumerate() {
        // TODO: color challenger gold, GM red and M pink
        table.add_row(row![format!("{}.", i + 1), pro, rank]);
    }
    table.printstd();
}
