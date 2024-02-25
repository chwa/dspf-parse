use bytesize::ByteSize;
use crossterm::event::KeyCode;
use ratatui::{prelude::*, widgets::*};

use crate::{app::Action, dspf::Dspf, event::Event};

use super::Render;

pub struct MainMenuUI {
    pub filename: String,
    pub filesize: u64,
    pub num_nets: usize,
    pub num_nodes: usize,
    pub num_elements: usize,
    menu: ListSelect,
}

impl MainMenuUI {
    pub fn new(path: &str, dspf: &Dspf) -> Self {
        Self {
            filename: path.to_owned(),
            filesize: u64::try_from(dspf.file_contents.len()).unwrap(),
            num_nets: dspf.netlist.as_ref().unwrap().all_nets.len(),
            num_nodes: dspf.netlist.as_ref().unwrap().all_nodes.len(),
            num_elements: dspf.netlist.as_ref().unwrap().all_parasitics.len(),
            menu: ListSelect::new(vec![
                "Report capacitance for net...".to_string(),
                "Report capacitance between 2 nets...".to_string(),
                "Report path resistance...".to_string(),
                "Quit".to_string(),
            ]),
        }
    }
}
impl Render for MainMenuUI {
    fn render(&mut self, frame: &mut Frame) -> () {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(7), Constraint::Fill(1)])
            .split(frame.size());

        let pad = |s| format!("{:<20}", s);
        let text = vec![
            Line::from(vec![
                Span::raw(pad("Filename:")),
                Span::styled(&self.filename, Style::new().bold()),
            ]),
            Line::from(vec![
                Span::raw(pad("Size:")),
                Span::styled(ByteSize(self.filesize).to_string(), Style::new().gray()),
            ]),
            Line::from(vec![
                Span::raw(pad("Nets:")),
                Span::styled(self.num_nets.to_string(), Style::new().gray()),
            ]),
            Line::from(vec![
                Span::raw(pad("Subnodes:")),
                Span::styled(self.num_nodes.to_string(), Style::new().gray()),
            ]),
            Line::from(vec![
                Span::raw(pad("Parasitic elements:")),
                Span::styled(self.num_elements.to_string(), Style::new().gray()),
            ]),
        ];

        frame.render_widget(
            Paragraph::new(text).block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(2)),
            ),
            layout[0],
        );

        let menu = List::new(self.menu.items.iter().map(AsRef::<str>::as_ref))
            .block(
                Block::default()
                    .title("Select:")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        // hack, how do I do this...
        self.menu
            .state
            .select(Some(self.menu.state.selected().unwrap_or(0)));
        frame.render_stateful_widget(menu, layout[1], &mut self.menu.state);
    }
    fn handle_event(&mut self, event: &Event) -> Action {
        match event {
            Event::Tick => Action::None,
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Up => {
                            self.menu.up(1);
                            Action::None
                        }
                        KeyCode::Down => {
                            self.menu.down(1);
                            Action::None
                        }
                        KeyCode::Enter => self.menu.select(),
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

pub struct ListSelect {
    pub state: ListState,
    pub items: Vec<String>,
}

impl ListSelect {
    pub fn new(items: Vec<String>) -> ListSelect {
        ListSelect {
            state: ListState::default(),
            items: items,
        }
    }

    pub fn down(&mut self, amount: usize) -> usize {
        let mut index = self.state.selected().unwrap_or(0);
        index = (index + amount).min(self.items.len() - 1);
        self.state.select(Some(index));
        index
    }

    pub fn up(&mut self, amount: usize) -> usize {
        let mut index = self.state.selected().unwrap_or(0);
        if index < amount {
            index = 0
        } else {
            index = index - amount
        }
        self.state.select(Some(index));
        index
    }

    pub fn select(&self) -> Action {
        match self.state.selected() {
            // Some(123) => Action::Select(i),
            Some(i) => Action::SelectMenuOption(i),
            None => Action::None,
        }
    }
}
