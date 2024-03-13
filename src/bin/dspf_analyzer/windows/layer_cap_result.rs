use crate::util::{eng_format_scale, line_bar};
use crate::{app::Action, event::Event};
use dspf_parse::dspf::netlist::LayerCapReport;
use ratatui::{prelude::*, widgets::*};

use super::net_cap_main::focus_style;

pub struct LayerCapResultWidget {
    pub focus: bool,
    report: LayerCapReport,
}

impl LayerCapResultWidget {
    pub fn new(report: LayerCapReport) -> Self {
        let ui = Self {
            focus: false,
            report,
        };
        ui
    }
}

impl Widget for &mut LayerCapResultWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(2), Constraint::Fill(1)])
            .split(area);

        let fs = focus_style(self.focus);

        Paragraph::new("\n  Layer pairs:").style(fs.1).render(rows_layout[0], buf);

        let rows: Vec<_> = self
            .report
            .table
            .iter()
            .map(|x| {
                let col1 = Line::raw(&x.layer_names.0);
                let col2 = Line::raw(&x.layer_names.1);
                let col3 = Line::raw(eng_format_scale(x.cap, self.report.total_cap));
                let col4 = line_bar(8, x.cap / self.report.total_cap);
                let col5 = Line::raw(format!("{:6.1}%", 100.0 * x.cap / self.report.total_cap));
                Row::new(vec![col1, col2, col3, col4, col5])
            })
            .collect();

        let widths = [
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(11),
            Constraint::Length(10),
            Constraint::Length(7),
        ];
        let table = Table::new(rows, widths).block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(fs.0)
                .padding(Padding::horizontal(1)),
        );
        Widget::render(table, rows_layout[1], buf);
    }
}
impl LayerCapResultWidget {
    pub fn handle_event(&mut self, event: &Event) -> Action {
        match event {
            Event::Tick => Action::None,
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    match key_event.code {
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
