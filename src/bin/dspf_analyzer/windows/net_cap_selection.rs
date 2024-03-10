use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::{NetInfo, NetType};
use globset::Glob;
use ratatui::{prelude::*, widgets::*};

use crate::app::Action;
use crate::event::Event;

use super::main_menu::ListSelect;
use super::net_cap_main::focus_style;

pub struct NetSelectionWidget {
    pub focus: bool,
    pub nets: Vec<NetInfo>,
    pub search_string: String,
    menu: ListSelect<NetInfo>,
    menu_height: u16,
}

impl NetSelectionWidget {
    pub fn new(mut nets: Vec<NetInfo>) -> Self {
        nets.sort_by_key(|info| (info.net_type.clone(), info.name.clone()));
        let mut ui = Self {
            focus: false,
            nets: nets,
            search_string: String::from("*"),
            menu: ListSelect::new(vec![]),
            menu_height: 1,
        };

        ui.update_list();
        ui
    }

    fn update_list(&mut self) {
        let glob = Glob::new(&self.search_string);
        let filtered: Vec<NetInfo> = match glob {
            Ok(g) => {
                let matcher = g.compile_matcher();
                let mut nets: Vec<NetInfo> =
                    self.nets.iter().filter(|net| matcher.is_match(&net.name)).cloned().collect();
                nets.sort_by_key(|info| (info.net_type.clone(), info.name.clone()));
                nets
            }
            Err(_) => Vec::new(),
        };

        let selection = match filtered.is_empty() {
            true => None,
            false => Some(0),
        };
        self.menu = ListSelect::new(filtered);
        self.menu.select_state(selection);
    }

    fn backspace(&mut self) {
        self.search_string.pop();
        self.update_list();
    }

    fn search_char(&mut self, c: char) {
        self.search_string.push(c);
        self.update_list();
    }
}

impl Widget for &mut NetSelectionWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(3),
            ])
            .split(area);

        let fs = focus_style(self.focus);

        Paragraph::new("\n  Victim net:").style(fs.1).render(rows_layout[0], buf);

        let list = List::new(self.menu.items.iter().map(|net| match net.net_type {
            NetType::GroundNode => format!(" ⏚  {}", net.name),
            NetType::SubcktPin => format!(" ⎔  {}", net.name),
            NetType::Other => format!("    {}", net.name),
        }))
        .block(Block::new().borders(Borders::ALL).border_type(fs.0))
        .highlight_style(Style::new().reversed());

        self.menu_height = rows_layout[1].as_size().height - 2;
        StatefulWidget::render(list, rows_layout[1], buf, &mut self.menu.state);

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
impl NetSelectionWidget {
    fn handle_arrow(&mut self, code: KeyCode) -> Action {
        let pos = match code {
            KeyCode::Up => self.menu.up(1),
            KeyCode::Down => self.menu.down(1),
            KeyCode::PageUp => self.menu.up((self.menu_height - 1).into()),
            KeyCode::PageDown => self.menu.down((self.menu_height - 1).into()),
            _ => 0, // not possible
        };
        let net_name = self.menu.items[pos].name.to_owned();
        Action::SelectNet(net_name)
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
                        KeyCode::Esc => Action::Esc,
                        KeyCode::Backspace => {
                            self.backspace();
                            match self.menu.state.selected() {
                                Some(idx) => {
                                    let net_name = self.menu.items[idx].name.to_owned();
                                    Action::SelectNet(net_name)
                                }
                                None => Action::None,
                            }
                        }
                        KeyCode::Char(c) => {
                            self.search_char(c);
                            if let Some(idx) = self.menu.state.selected() {
                                let net_name = self.menu.items[idx].name.to_owned();
                                return Action::SelectNet(net_name);
                            }
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
