use crate::util::{eng_format_scale, line_bar};
use crate::{app::Action, event::Event};
use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::{AggrNet, NetCapForAggressor, NetCapReport};
use globset::Glob;
use ratatui::{prelude::*, widgets::*};

use super::main_menu::TableSelect;
use super::net_cap_main::focus_style;

pub struct NetCapResultWidget {
    pub focus: bool,
    report: NetCapReport,
    pub search_string: String,
    pub menu: TableSelect<NetCapForAggressor>,
    menu_height: u16,
}

impl NetCapResultWidget {
    pub fn new(report: NetCapReport) -> Self {
        let mut ui = Self {
            focus: false,
            report,
            search_string: String::from("*"),
            menu: TableSelect::new(vec![]),
            menu_height: 1,
        };

        ui.update_list();
        ui
    }

    pub fn selected(&self) -> Option<AggrNet> {
        self.menu.selected().map(|s| s.aggressor.clone())
    }

    fn update_list(&mut self) -> Action {
        let glob = Glob::new(&self.search_string);

        let mut aggressors_filtered: Vec<_> = match glob {
            Ok(g) => {
                let matcher = g.compile_matcher();
                self.report
                    .table
                    .iter()
                    .filter(|item| match item {
                        NetCapForAggressor {
                            aggressor: AggrNet::Net(net_name),
                            ..
                        } => matcher.is_match(net_name),
                        _ => true,
                    })
                    .cloned()
                    .collect()
            }
            Err(_) => {
                vec![]
            }
        };

        aggressors_filtered.insert(0, self.report.total_cap.clone());

        self.menu = TableSelect::new(aggressors_filtered);

        if self.menu.items.len() > 0 {
            self.menu.select_state(Some(0));
        }

        Action::SelectAggrNet(self.selected())
    }
    fn handle_backspace(&mut self) -> Action {
        self.search_string.pop();
        self.update_list()
    }

    fn handle_search_char(&mut self, c: char) -> Action {
        self.search_string.push(c);
        self.update_list()
    }

    fn handle_arrow(&mut self, code: KeyCode) -> Action {
        match code {
            KeyCode::Up => self.menu.up(1),
            KeyCode::Down => self.menu.down(1),
            KeyCode::PageUp => self.menu.up((self.menu_height - 1).into()),
            KeyCode::PageDown => self.menu.down((self.menu_height - 1).into()),
            _ => 0, // not possible
        };

        Action::SelectAggrNet(self.selected())
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
                        KeyCode::Backspace => self.handle_backspace(),
                        KeyCode::Char(c) => self.handle_search_char(c),
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
impl Widget for &mut NetCapResultWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(3),
            ])
            .split(area);
        self.menu_height = rows_layout[1].as_size().height - 2;
        let fs = focus_style(self.focus);

        Paragraph::new("\n  Aggressor net:").style(fs.1).render(rows_layout[0], buf);

        let total_c = self.report.total_cap.cap;

        let rows: Vec<_> = self
            .menu
            .items
            .iter()
            .map(|x| {
                let col1 = Line::raw(x.aggressor.to_string());
                let col2 = Line::raw(eng_format_scale(x.cap, total_c));
                let mut col3 = line_bar(8, x.cap / total_c);
                let col4 = Line::raw(format!("{:6.1}%", 100.0 * x.cap / total_c));
                let mut sty = Style::new();
                if let AggrNet::Total = x.aggressor {
                    sty = sty.bold();
                    col3 = Line::raw("");
                }

                Row::new(vec![col1, col2, col3, col4]).style(sty)
            })
            .collect();

        let widths = [
            Constraint::Fill(2),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(7),
        ];

        StatefulWidget::render(
            Table::new(rows, widths)
                .block(
                    Block::new()
                        .borders(Borders::ALL)
                        .border_type(fs.0)
                        .padding(Padding::horizontal(1)),
                )
                .highlight_style(Style::new().reversed()),
            rows_layout[1],
            buf,
            &mut self.menu.state,
        );

        Paragraph::new(Span::from(&self.search_string))
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(1)),
            )
            .render(rows_layout[2], buf);
    }
}
