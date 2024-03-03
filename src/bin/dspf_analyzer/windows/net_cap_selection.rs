use crate::{app::Action, event::Event};
use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::NetCapReport;
use dspf_parse::dspf::Dspf;
use globset::Glob;
use ratatui::Frame;
use ratatui::{prelude::*, widgets::*};
use std::char;

use super::net_cap_result::NetCapResultUI;
use super::{main_menu::ListSelect, Render};

pub struct NetCapSelectionUI {
    pub filename: String,
    pub net_names: Vec<String>,
    pub selected_net: Option<String>,
    pub search_string: String,
    menu: ListSelect,
    pub result_ui: NetCapResultUI,
}

impl NetCapSelectionUI {
    pub fn new(path: &str, dspf: &Dspf) -> Self {
        let mut net_names: Vec<String> = dspf
            .netlist
            .as_ref()
            .unwrap()
            .nets_map
            .keys()
            .cloned()
            .collect();
        net_names.sort();
        let mut ui = Self {
            filename: path.to_owned(),
            net_names: net_names,
            selected_net: None,
            search_string: String::from("*"),
            menu: ListSelect::new(vec![]),
            result_ui: NetCapResultUI::new(NetCapReport::default()),
        };

        ui.search_changed();

        ui
    }
    fn selection_changed(&mut self, selection: Option<usize>) {
        self.selected_net = match selection {
            Some(i) => self.menu.items.get(i).map(String::from),
            None => None,
        }
        // self.selected_net = self.menu.items.get(i).unwrap().to_owned();
    }

    fn search_changed(&mut self) {
        let glob = Glob::new(&self.search_string);
        let filtered: Vec<_> = match glob {
            Ok(g) => {
                let matcher = g.compile_matcher();
                self.net_names
                    .iter()
                    .filter(|s| matcher.is_match(s))
                    .cloned()
                    .collect()
            }
            Err(_) => Vec::new(),
        };

        let selection = match filtered.is_empty() {
            true => None,
            false => Some(0),
        };
        self.menu = ListSelect::new(filtered);
        self.menu.select_state(selection);
        self.selection_changed(selection);
    }

    fn backspace(&mut self) {
        self.search_string.pop();
        self.search_changed();
    }

    fn search_char(&mut self, c: char) {
        self.search_string.push(c);
        self.search_changed();
    }
}

impl Render for NetCapSelectionUI {
    fn render(&mut self, frame: &mut Frame) -> () {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Fill(1)])
            .split(frame.size());
        let x = self.selected_net.clone().or(Some(String::new()));
        let text = Line::from(vec![
            Span::styled("Selected: ", Style::new().bold()),
            Span::raw(x.unwrap()),
        ]);

        frame.render_widget(
            Paragraph::new(text).block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(2)),
            ),
            rows_layout[0],
        );

        let cols_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Fill(2),
                // Constraint::Length(3),
            ])
            .split(rows_layout[1]);

        let inner_rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Length(3)])
            .split(cols_layout[0]);

        let menu = List::new(self.menu.items.iter().map(AsRef::<str>::as_ref))
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        // hack, how do I do this...
        self.menu
            .state
            .select(Some(self.menu.state.selected().unwrap_or(0)));
        frame.render_stateful_widget(menu, inner_rows_layout[0], &mut self.menu.state);

        frame.render_widget(
            Paragraph::new(Span::from(&self.search_string)).block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(2)),
            ),
            inner_rows_layout[1],
        );
        self.result_ui.render_in_rect(frame, &cols_layout[1]);
    }
    fn handle_event(&mut self, event: &Event) -> Action {
        match event {
            Event::Tick => Action::None,
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Up => {
                            let pos = self.menu.up(1);
                            self.selection_changed(Some(pos));
                            let net_name = self.menu.items[pos].to_owned();
                            Action::SelectNet(net_name)
                        }
                        KeyCode::Down => {
                            let pos = self.menu.down(1);
                            self.selection_changed(Some(pos));
                            let net_name = self.menu.items[pos].to_owned();
                            Action::SelectNet(net_name)
                        }
                        KeyCode::PageUp => {
                            let pos = self.menu.up(10);
                            self.selection_changed(Some(pos));
                            let net_name = self.menu.items[pos].to_owned();
                            Action::SelectNet(net_name)
                        }
                        KeyCode::PageDown => {
                            let pos = self.menu.down(10);
                            self.selection_changed(Some(pos));
                            let net_name = self.menu.items[pos].to_owned();
                            Action::SelectNet(net_name)
                        }
                        KeyCode::Enter => {
                            let idx = self.menu.state.selected().unwrap();
                            let net_name = self.menu.items[idx].to_owned();
                            Action::SelectNet(net_name)
                        }
                        KeyCode::Esc => Action::Quit,
                        KeyCode::Backspace => {
                            self.backspace();
                            let idx = self.menu.state.selected().unwrap();
                            let net_name = self.menu.items[idx].to_owned();
                            Action::SelectNet(net_name)
                        }
                        KeyCode::Char(c) => {
                            self.search_char(c);
                            if let Some(idx) = self.menu.state.selected() {
                                let net_name = self.menu.items[idx].to_owned();
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
