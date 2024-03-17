use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::{prelude::*, widgets::*};

#[derive(Default)]
pub struct StatusBar {
    top: (String, String),
    bottom: (String, String),
    pub inner: Rect,
}

impl StatusBar {
    pub fn top_left(mut self, a: &str) -> Self {
        self.top.0 = a.to_owned();
        self
    }
    pub fn top_right(mut self, a: &str) -> Self {
        self.top.1 = a.to_owned();
        self
    }
    pub fn bottom_left(mut self, a: &str) -> Self {
        self.bottom.0 = a.to_owned();
        self
    }
    pub fn bottom_right(mut self, a: &str) -> Self {
        self.bottom.1 = a.to_owned();
        self
    }
}

impl Widget for &mut StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .split(area);

        self.inner = rows_layout[1];

        let cols_top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
            .split(rows_layout[0]);

        Paragraph::new(self.top.0.clone())
            .style(Style::new().add_modifier(Modifier::REVERSED))
            .render(cols_top[0], buf);
        Paragraph::new(self.top.1.clone())
            .style(Style::new().add_modifier(Modifier::REVERSED))
            .render(cols_top[1], buf);

        let cols_bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
            .split(rows_layout[2]);

        Paragraph::new(self.bottom.0.clone())
            .style(Style::new().add_modifier(Modifier::REVERSED))
            .render(cols_bottom[0], buf);
        Paragraph::new(self.bottom.1.clone())
            .style(Style::new().add_modifier(Modifier::REVERSED))
            .render(cols_bottom[1], buf);
    }
}
