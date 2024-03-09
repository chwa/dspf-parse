use crate::util::eng_format;
use crate::{app::Action, event::Event};
use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::LayerCapReport;
use ratatui::Frame;
use ratatui::{prelude::*, widgets::*};

use super::net_cap_result::line_bar;
use super::Render;

pub struct LayerCapResultUI {
    report: LayerCapReport,
}

impl LayerCapResultUI {
    pub fn new(report: LayerCapReport) -> Self {
        let ui = Self { report };
        ui
    }

    pub fn render_in_rect(&mut self, frame: &mut Frame, rect: &Rect) -> () {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(2), Constraint::Fill(1)])
            .split(*rect);

        frame.render_widget(Paragraph::new("\n  Layer pairs:"), rows_layout[0]);

        let rows: Vec<_> = self
            .report
            .table
            .iter()
            .map(|x| {
                let col1 = Line::raw(&x.layer_names.0);
                let col2 = Line::raw(&x.layer_names.1);
                let col3 = Line::raw(eng_format(x.cap));
                let col4 = line_bar(8, x.cap / self.report.total_cap);
                let col5 = Line::raw(format!("{:6.1}%", 100.0 * x.cap / self.report.total_cap));
                Row::new(vec![col1, col2, col3, col4, col5])
            })
            .collect();

        let widths = [
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(11),
            Constraint::Length(8),
            Constraint::Length(7),
        ];
        let table = Table::new(rows, widths).block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .padding(Padding::horizontal(1)),
        );
        frame.render_widget(table, rows_layout[1]);
    }
}

impl Render for LayerCapResultUI {
    fn render(&mut self, frame: &mut Frame) -> () {
        self.render_in_rect(frame, &frame.size());
    }

    fn handle_event(&mut self, event: &Event) -> Action {
        match event {
            Event::Tick => Action::None,
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => Action::Quit,

                        _ => Action::None,
                    }
                } else {
                    Action::None
                }
            }
            Event::Mouse(_) => Action::None,
            Event::Resize(_, _) => Action::None,
        }
    }
}
