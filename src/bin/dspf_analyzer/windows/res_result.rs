use std::f64::NAN;

use crate::{
    app::Action,
    event::Event,
    util::{eng_format_res, line_bar},
};

use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::{NodeResistance, ResForLayer, ResReport};

use ratatui::{prelude::*, widgets::*};

use super::{main_menu::TableSelect, net_cap_main::focus_style};

#[derive(Default)]
pub struct ResResultWidget {
    pub focus: bool,
    report: ResReport,
    pub output_list: TableSelect<NodeResistance>,
    pub layer_list: TableSelect<ResForLayer>,
    menu_height: u16,
}

impl ResResultWidget {
    pub fn new(report: ResReport) -> Self {
        let mut table_outputs_sorted = report.table_outputs.clone();
        table_outputs_sorted.sort_by(|a, b| b.resistance.total_cmp(&a.resistance));

        let mut output_list = TableSelect::new(table_outputs_sorted);

        if !output_list.items.is_empty() {
            output_list.select_state(Some(0));
        }

        let mut table_layers_sorted = report.table_layers.clone();
        table_layers_sorted.sort_by(|a, b| b.res.total_cmp(&a.res));

        let mut layer_list = TableSelect::new(table_layers_sorted);

        if !layer_list.items.is_empty() {
            layer_list.select_state(Some(0));
        }

        Self {
            focus: false,
            report,
            output_list,
            layer_list,
            menu_height: 1,
        }
    }
    fn handle_arrow(&mut self, code: KeyCode) -> Action {
        match code {
            KeyCode::Up => self.output_list.up(1),
            KeyCode::Down => self.output_list.down(1),
            KeyCode::PageUp => self.output_list.up((self.menu_height - 1).into()),
            KeyCode::PageDown => self.output_list.down((self.menu_height - 1).into()),
            _ => 0, // not possible
        };

        Action::None
    }
    pub fn handle_event(&mut self, event: &Event) -> Action {
        match event {
            Event::Tick => Action::None,
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown => {
                            self.handle_arrow(key_event.code)
                        }
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
impl Widget for &mut ResResultWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(area);

        self.menu_height = rows_layout[1].as_size().height - 2;

        let fs = focus_style(self.focus);

        Paragraph::new(format!(
            "\n  Total effective R: {}",
            eng_format_res(self.report.total_res, self.report.total_res),
        ))
        .render(rows_layout[0], buf);

        Paragraph::new("\n  Effective R for output port: [~IR drop]")
            .style(fs.1)
            .render(rows_layout[1], buf);

        let max_r = self
            .output_list
            .items
            .iter()
            .max_by(|a, b| a.resistance.total_cmp(&b.resistance))
            .map(|x| x.resistance)
            .unwrap_or(NAN);

        let rows: Vec<_> = self
            .output_list
            .items
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
        let table = Table::new(rows, widths)
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(fs.0)
                    .padding(Padding::horizontal(1)),
            )
            .highlight_style(Style::new().reversed());
        StatefulWidget::render(table, rows_layout[2], buf, &mut self.output_list.state);

        Paragraph::new("  Layer contributions to total R:").render(rows_layout[3], buf);

        // --layers
        let rows: Vec<_> = self
            .layer_list
            .items
            .iter()
            .map(|x| {
                let col1 = Line::raw(&x.layer_name);
                let col2 = Line::raw(eng_format_res(x.res, max_r));
                let col3 = line_bar(12, x.res / self.report.total_res);
                let col4 = Line::raw(format!("{:5.1}%", 100.0 * x.res / self.report.total_res));
                Row::new(vec![col1, col2, col3, col4])
            })
            .collect();

        let widths = [
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(6),
        ];
        let table = Table::new(rows, widths).block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .padding(Padding::horizontal(1)),
        );
        StatefulWidget::render(table, rows_layout[4], buf, &mut self.layer_list.state);
    }
}
