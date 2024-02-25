pub mod main_menu;
pub mod net_cap;

use ratatui::{
    layout::Alignment,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{app::Action, event::Event};

use self::main_menu::MainMenuUI;
use self::net_cap::NetCapUI;

pub trait Render {
    fn render(&mut self, frame: &mut Frame) -> ();
    fn handle_event(&mut self, event: &Event) -> Action;
}

pub enum UI {
    Blank(BlankUI),
    MainMenu(MainMenuUI),
    NetCap(NetCapUI),
    Simple(SimpleUI),
}

impl UI {
    pub fn blank() -> Self {
        UI::Blank(BlankUI {})
    }
}

impl Render for UI {
    fn render(&mut self, frame: &mut Frame) -> () {
        match self {
            UI::Blank(ui) => ui.render(frame),
            UI::MainMenu(ui) => ui.render(frame),
            UI::NetCap(ui) => ui.render(frame),
            UI::Simple(ui) => ui.render(frame),
        }
    }
    fn handle_event(&mut self, event: &Event) -> Action {
        match self {
            UI::Blank(ui) => ui.handle_event(event),
            UI::MainMenu(ui) => ui.handle_event(event),
            UI::NetCap(ui) => ui.handle_event(event),
            UI::Simple(ui) => ui.handle_event(event),
        }
    }
}

/// Examples of possible UIs

pub struct BlankUI {}
impl Render for BlankUI {
    fn render(&mut self, _: &mut Frame) -> () {}
    fn handle_event(&mut self, _: &Event) -> Action {
        Action::None
    }
}

pub struct SimpleUI {
    pub text: String,
}

impl SimpleUI {
    pub fn new() -> Self {
        Self {
            text: String::from("Loading..."),
        }
    }
}

impl Render for SimpleUI {
    fn render(&mut self, frame: &mut Frame) -> () {
        frame.render_widget(
            Paragraph::new(self.text.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .alignment(Alignment::Center),
            frame.size(),
        )
    }

    fn handle_event(&mut self, event: &Event) -> Action {
        match event {
            Event::Tick => Action::None,
            Event::Key(_) => Action::Quit,
            Event::Mouse(_) => Action::None,
            Event::Resize(_, _) => Action::None,
        }
    }
}
