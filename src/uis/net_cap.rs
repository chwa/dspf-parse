use std::char;

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::{prelude::*, widgets::*};

use crate::{app::Action, dspf::Dspf, event::Event};

use super::{main_menu::ListSelect, Render};

pub struct NetCapUI {
    pub filename: String,
    pub net_names: Vec<String>,
    pub selected_net: String,
    pub search_string: String,
    menu: ListSelect,
}

impl NetCapUI {
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
            selected_net: String::new(),
            search_string: String::new(),
            menu: ListSelect::new(vec![]),
        };
        ui.search_changed();
        ui.selection_changed(0);

        ui
    }
    fn selection_changed(&mut self, i: usize) {
        self.selected_net = self.menu.items.get(i).unwrap().to_owned();
    }

    fn search_changed(&mut self) {
        let filtered = self
            .net_names
            .iter()
            .filter(|s| s.contains(&self.search_string))
            .cloned()
            .collect();
        self.menu = ListSelect::new(filtered);
        self.selection_changed(0);
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

impl Render for NetCapUI {
    fn render(&mut self, frame: &mut Frame) -> () {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(3),
            ])
            .split(frame.size());

        let text = Line::from(vec![
            Span::styled("Selected: ", Style::new().bold()),
            Span::raw(&self.selected_net),
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

        let menu = List::new(self.menu.items.iter().map(AsRef::<str>::as_ref))
            .block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_widget(
            Paragraph::new(Span::from(&self.search_string)).block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(2)),
            ),
            rows_layout[2],
        );

        // hack, how do I do this...
        self.menu
            .state
            .select(Some(self.menu.state.selected().unwrap_or(0)));
        frame.render_stateful_widget(menu, rows_layout[1], &mut self.menu.state);
    }
    fn handle_event(&mut self, event: &Event) -> Action {
        match event {
            Event::Tick => Action::None,
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Up => {
                            let pos = self.menu.up(1);
                            self.selection_changed(pos);
                            Action::None
                        }
                        KeyCode::Down => {
                            let pos = self.menu.down(1);
                            self.selection_changed(pos);
                            Action::None
                        }
                        KeyCode::PageUp => {
                            let pos = self.menu.up(10);
                            self.selection_changed(pos);
                            Action::None
                        }
                        KeyCode::PageDown => {
                            let pos = self.menu.down(10);
                            self.selection_changed(pos);
                            Action::None
                        }
                        KeyCode::Enter => self.menu.select(),
                        KeyCode::Esc => Action::Quit,
                        KeyCode::Backspace => {
                            self.backspace();
                            Action::None
                        }
                        KeyCode::Char(c) => {
                            self.search_char(c);
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
