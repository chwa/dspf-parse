use std::f64::NAN;

use crate::util::{eng_format_res, line_bar};

use dspf_parse::dspf::netlist::ResReport;

use ratatui::{prelude::*, widgets::*};

use super::net_cap_main::focus_style;

#[derive(Default)]
pub struct ResResultWidget {
    pub focus: bool,
    report: ResReport,
}

impl ResResultWidget {
    pub fn new(report: ResReport) -> Self {
        Self {
            focus: false,
            report,
        }
    }
}
impl Widget for &mut ResResultWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(2), Constraint::Fill(1)])
            .split(area);

        let fs = focus_style(self.focus);

        Paragraph::new("\n  Resistance:").style(fs.1).render(rows_layout[0], buf);

        let max_r = self
            .report
            .table
            .iter()
            .max_by(|a, b| a.resistance.total_cmp(&b.resistance))
            .map(|x| x.resistance)
            .unwrap_or(NAN);

        let rows: Vec<_> = self
            .report
            .table
            .iter()
            .map(|x| {
                let col1 = Line::raw(&x.node);
                let col2 = Line::raw(eng_format_res(x.resistance, max_r));
                let col3 = line_bar(12, x.resistance / max_r);
                Row::new(vec![col1, col2, col3])
            })
            .collect();

        let widths = [
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(12),
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
