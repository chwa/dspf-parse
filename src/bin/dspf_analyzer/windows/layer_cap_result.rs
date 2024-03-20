use crate::util::{eng_format_cap, line_bar};
use crate::{app::Action, event::Event};
use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::{LayerCapGroupBy, LayerCapReport};
use ratatui::{prelude::*, widgets::*};

use super::net_cap_main::focus_style;

pub struct LayerCapResultWidget {
    pub focus: bool,
    report: LayerCapReport,
    view_mode: LayerCapViewMode,
}

impl LayerCapResultWidget {
    pub fn new(report: LayerCapReport) -> Self {
        Self {
            focus: false,
            report,
            view_mode: LayerCapViewMode::Flat,
        }
    }
    fn change_view(&mut self) {
        use LayerCapGroupBy::*;
        use LayerCapViewMode::*;
        self.view_mode = match self.view_mode {
            Flat => Grouped(VictimLayer),
            Grouped(VictimLayer) => Grouped(AggrLayer),
            Grouped(AggrLayer) => Flat,
        }
    }
}

enum LayerCapViewMode {
    Flat,
    Grouped(LayerCapGroupBy),
}

impl Widget for &mut LayerCapResultWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(2), Constraint::Fill(1)])
            .split(area);

        let fs = focus_style(self.focus);

        Paragraph::new("\n  Layer pairs:").style(fs.1).render(rows_layout[0], buf);

        let mut rows: Vec<Row> = vec![
            Row::new(vec![
                Span::styled("self:", Style::new().bold()),
                Span::styled("other:", Style::new().bold()),
            ]),
            Row::new(vec![""]),
        ];

        // TODO: rows should not be re-computed inside render()

        match self.view_mode {
            LayerCapViewMode::Flat => {
                rows.extend(self.report.table.iter().map(|x| {
                    let col1 = Line::raw(&x.layer_names.0);
                    let col2 = Line::raw(&x.layer_names.1);
                    let col3 = Line::raw(eng_format_cap(x.cap, self.report.total_cap));
                    let col4 = line_bar(12, x.cap / self.report.total_cap);
                    let col5 = Line::raw(format!("{:5.1}%", 100.0 * x.cap / self.report.total_cap));
                    Row::new(vec![col1, col2, col3, col4, col5])
                }));
            }
            LayerCapViewMode::Grouped(group_by) => {
                let grouped = self.report.grouped(group_by);
                for item in grouped {
                    let layer = Line::raw(item.layer.to_owned());
                    let mut row = match group_by {
                        LayerCapGroupBy::VictimLayer => vec![layer, Line::default()],
                        LayerCapGroupBy::AggrLayer => vec![Line::default(), layer],
                    };
                    row.push(Line::raw(eng_format_cap(
                        item.total_cap,
                        self.report.total_cap,
                    )));
                    row.push(line_bar(12, item.total_cap / self.report.total_cap));
                    row.push(Line::raw(format!(
                        "{:5.1}%",
                        100.0 * item.total_cap / self.report.total_cap
                    )));
                    rows.push(Row::new(row));
                    let mut items = item.individual.iter().peekable();
                    while let Some(second) = items.next() {
                        let mut row = match group_by {
                            LayerCapGroupBy::VictimLayer => {
                                vec![
                                    Line::raw(match items.peek() {
                                        Some(_) => String::from("├") + &"─".repeat(40),
                                        None => String::from("└") + &"─".repeat(40),
                                    }),
                                    Line::raw(second.0.clone()),
                                ]
                            }
                            LayerCapGroupBy::AggrLayer => {
                                vec![
                                    Line::raw(second.0.clone() + " " + &"─".repeat(40)),
                                    Line::raw(match items.peek() {
                                        Some(_) => String::from("┤"),
                                        None => String::from("┘"),
                                    }),
                                ]
                            }
                        };
                        row.push(Line::raw(eng_format_cap(second.1, self.report.total_cap)));
                        row.push(line_bar(12, second.1 / self.report.total_cap));
                        row.push(Line::raw(format!(
                            "{:5.1}%",
                            100.0 * second.1 / self.report.total_cap
                        )));
                        rows.push(Row::new(row));
                    }
                    rows.push(Row::new(vec![Cell::from(" ")]));
                }
            }
        }

        let widths = [
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(6),
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
                        KeyCode::Char(' ') => {
                            self.change_view();
                            Action::None
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
