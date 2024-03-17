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
    pub menu: ListSelect<NetInfo>,
    title: String,
    menu_height: u16,
    enter_to_select: bool,
}

impl NetSelectionWidget {
    pub fn new(mut nets: Vec<NetInfo>, title: &str, enter_to_select: bool) -> Self {
        nets.sort_by_key(|info| (info.net_type.clone(), info.name.clone()));
        let mut ui = Self {
            focus: false,
            nets,
            search_string: String::from("*"),
            menu: ListSelect::new(vec![]),
            title: title.to_owned(),
            menu_height: 1,
            enter_to_select,
        };

        ui.update_list();
        ui
    }

    pub fn selected(&self) -> Option<String> {
        self.menu.state.selected().map(|pos| self.menu.items[pos].name.clone())
    }

    pub fn update_list(&mut self) -> Action {
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

        Action::SelectNet(selection.map(|pos| self.menu.items[pos].name.clone()))
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
        let pos = match code {
            KeyCode::Up => self.menu.up(1),
            KeyCode::Down => self.menu.down(1),
            KeyCode::PageUp => self.menu.up((self.menu_height - 1).into()),
            KeyCode::PageDown => self.menu.down((self.menu_height - 1).into()),
            _ => 0, // not possible
        };
        let net_name = self.menu.items[pos].name.clone();
        match self.enter_to_select {
            false => Action::SelectNet(Some(net_name)),
            true => Action::None,
        }
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
                        KeyCode::Enter => match self.menu.state.selected() {
                            Some(pos) => {
                                let net_name = self.menu.items[pos].name.clone();
                                Action::SelectNet(Some(net_name))
                            }
                            None => Action::None,
                        },
                        KeyCode::Esc => Action::Esc,
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

        Paragraph::new(format!("\n  {}", self.title))
            .style(fs.1)
            .render(rows_layout[0], buf);

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
