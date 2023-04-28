use crate::pro_data::ProData;
use prettytable::{color, format, row, Attr, Table};

pub fn print(pro_data: &ProData) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_CLEAN);
    let leaderboard = pro_data.pro_leaderboard();
    for (i, (pro, rank)) in leaderboard.iter().enumerate() {
        let mut row = row![format!("{}.", i + 1), pro, rank];
        let color = match rank.tier {
            riven::consts::Tier::CHALLENGER => color::BRIGHT_YELLOW,
            riven::consts::Tier::GRANDMASTER => color::RED,
            riven::consts::Tier::MASTER => color::MAGENTA,
            _ => panic!("Rank should never be below master"),
        };
        for cell in row.iter_mut() {
            cell.style(Attr::ForegroundColor(color))
        }
        table.add_row(row);
    }
    table.printstd();
}
