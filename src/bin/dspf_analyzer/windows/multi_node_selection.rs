use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::{Node, NodeType};
use globset::Glob;
use ratatui::{prelude::*, widgets::*};

use crate::app::Action;
use crate::event::Event;

use super::main_menu::ListSelect;
use super::net_cap_main::focus_style;

#[derive(Default)]
pub struct MultiNodeSelectionWidget {
    pub focus: bool,
    pub nodes: Vec<NodeInfo>,
    pub search_string: String,
    pub menu: ListSelect<NodeInfo>,
    title: String,
    menu_height: u16,
    excluded: Vec<NodeInfo>,
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DisplayNodeType {
    #[default]
    SubcktPin,
    InstPin,
}

// consider moving this to netlist.rs and using in Node struct?
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeInfo {
    pub name: String,
    pub node_type: DisplayNodeType,
}

impl MultiNodeSelectionWidget {
    pub fn new(nodes: Vec<&Node>, title: &str) -> Self {
        let mut nodes: Vec<_> = nodes
            .iter()
            .filter_map(|node| match &node.info {
                NodeType::SubcktPin {
                    pin_type: _,
                    pin_cap: _,
                } => Some(NodeInfo {
                    name: node.name.clone(),
                    node_type: DisplayNodeType::SubcktPin,
                }),
                NodeType::InstPin {
                    inst_name: _,
                    pin_name: _,
                    pin_type: _,
                    pin_cap: _,
                } => Some(NodeInfo {
                    name: node.name.clone(),
                    node_type: DisplayNodeType::InstPin,
                }),
                _ => None,
            })
            .collect();

        nodes.sort();
        let mut ui = Self {
            focus: false,
            nodes,
            search_string: String::from("*"),
            menu: ListSelect::new(vec![]),
            title: title.to_owned(),
            menu_height: 1,
            excluded: Vec::new(),
        };

        ui.update_list();
        ui
    }

    pub fn exclude(&mut self, nodes: Vec<NodeInfo>) {
        self.excluded = nodes;
        self.update_list();
    }

    pub fn selected(&self) -> Option<String> {
        self.menu.state.selected().map(|pos| self.menu.items[pos].name.clone())
    }

    pub fn update_list(&mut self) -> Action {
        let glob = Glob::new(&self.search_string);
        let filtered: Vec<NodeInfo> = match glob {
            Ok(g) => {
                let matcher = g.compile_matcher();
                let mut nodes: Vec<_> = self
                    .nodes
                    .iter()
                    .filter(|net| matcher.is_match(&net.name) && !self.excluded.contains(net))
                    .cloned()
                    .collect();
                nodes.sort_by_key(|info| (info.node_type.clone(), info.name.clone()));
                nodes
            }
            Err(_) => Vec::new(),
        };

        let selection = match filtered.is_empty() {
            true => None,
            false => Some(0),
        };
        self.menu = ListSelect::new(filtered);
        self.menu.select_state(selection);

        Action::NodesChanged
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
        let _pos = match code {
            KeyCode::Up => self.menu.up(1),
            KeyCode::Down => self.menu.down(1),
            KeyCode::PageUp => self.menu.up((self.menu_height - 1).into()),
            KeyCode::PageDown => self.menu.down((self.menu_height - 1).into()),
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
                        KeyCode::Esc => Action::MainMenu,
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

impl Widget for &mut MultiNodeSelectionWidget {
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

        let list = List::new(self.menu.items.iter().map(|node| match node.node_type {
            DisplayNodeType::SubcktPin => format!(" ⎔  {}", node.name),
            DisplayNodeType::InstPin => format!(" ◰  {}", node.name),
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
