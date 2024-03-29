use bytesize::ByteSize;
use crossterm::event::KeyCode;
use ratatui::{prelude::*, widgets::*};

use crate::{
    app::{Action, MainMenuOption},
    event::Event,
};
use dspf_parse::dspf::Dspf;

use super::{status_bar::StatusBar, Render};

pub struct MainMenuUI {
    pub filename: String,
    pub filesize: u64,
    pub num_nets: usize,
    pub num_nodes: usize,
    pub num_capacitors: usize,
    pub num_resistors: usize,
    menu: ListSelect<MainMenuOption>,
}

impl MainMenuUI {
    pub fn new(dspf: &Dspf, options: &[MainMenuOption]) -> Self {
        Self {
            filename: dspf.file_path.to_owned(),
            filesize: dspf.file_size,
            num_nets: dspf.netlist.all_nets.len(),
            num_nodes: dspf.netlist.all_nodes.len(),
            num_capacitors: dspf.netlist.capacitors.len(),
            num_resistors: dspf.netlist.all_nets.iter().map(|net| net.resistors.len()).sum(),
            menu: ListSelect::new(options.to_vec()),
        }
    }
}
impl Render for MainMenuUI {
    fn render(&mut self, frame: &mut Frame) {
        let mut status_bar =
            StatusBar::default().top_left("dspf-analyzer").bottom_left(&self.filename);
        frame.render_widget(&mut status_bar, frame.size());

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(8), Constraint::Fill(1)])
            .split(status_bar.inner);

        let pad = |s| format!("{:<24}", s);
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
                Span::raw(pad("Parasitic capacitors:")),
                Span::styled(self.num_capacitors.to_string(), Style::new().gray()),
            ]),
            Line::from(vec![
                Span::raw(pad("Parasitic resistors:")),
                Span::styled(self.num_resistors.to_string(), Style::new().gray()),
            ]),
        ];

        frame.render_widget(
            Paragraph::new(text).block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::horizontal(1)),
            ),
            layout[0],
        );

        let menu = List::new(self.menu.items.iter().map(|i| i.to_string()))
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        // hack, how do I do this...
        self.menu.state.select(Some(self.menu.state.selected().unwrap_or(0)));
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
                        KeyCode::Enter => match self.menu.state.selected() {
                            Some(idx) => Action::SelectMenuOption(self.menu.items[idx]),
                            None => Action::None,
                        },
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

#[derive(Default)]
pub struct ListSelect<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> ListSelect<T> {
    pub fn new(items: Vec<T>) -> Self {
        ListSelect {
            state: ListState::default(),
            items,
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
            index -= amount
        }
        self.state.select(Some(index));
        index
    }
    pub fn select_state(&mut self, state: Option<usize>) {
        self.state.select(state);
    }
}

#[derive(Default)]
pub struct TableSelect<T> {
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> TableSelect<T> {
    pub fn new(items: Vec<T>) -> Self {
        TableSelect {
            state: TableState::default(),
            items,
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
            index -= amount
        }
        self.state.select(Some(index));
        index
    }
    pub fn select_state(&mut self, state: Option<usize>) {
        self.state.select(state);
    }

    pub fn selected(&self) -> Option<&T> {
        self.state.selected().map(|idx| &self.items[idx])
    }
}
