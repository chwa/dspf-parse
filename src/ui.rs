use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{Alignment, Frame},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::App;

pub fn render(app: &mut App, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.size());
    f.render_widget(
        Paragraph::new(format!(
            "
        Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
        Press `j` and `k` to increment and decrement the counter respectively.\n\
        Counter: {}
      ",
            app.counter
        ))
        .block(
            Block::default()
                .title("Counter App")
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_type(BorderType::Double),
        )
        .style(
            Style::default()
                .fg(Color::Rgb(180, 220, 255))
                .bg(Color::Rgb(30, 30, 20)),
        )
        .alignment(Alignment::Center),
        chunks[0],
    )
}
