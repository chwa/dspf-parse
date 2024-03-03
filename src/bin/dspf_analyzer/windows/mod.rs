pub mod main_menu;
pub mod net_cap_result;
pub mod net_cap_selection;

use std::sync::{Arc, Mutex};

use ratatui::{
    layout::Alignment,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::{app::Action, event::Event};

use self::main_menu::MainMenuUI;
use self::net_cap_result::NetCapResultUI;
use self::net_cap_selection::NetCapSelectionUI;

pub trait Render {
    fn render(&mut self, frame: &mut Frame) -> ();
    fn handle_event(&mut self, event: &Event) -> Action;
}

pub enum Window {
    Blank(BlankUI),
    MainMenu(MainMenuUI),
    NetCapSelection(NetCapSelectionUI),
    NetCapResult(NetCapResultUI),
    Progress(ProgressUI),
}

impl Window {
    pub fn blank() -> Self {
        Window::Blank(BlankUI {})
    }
}

impl Render for Window {
    fn render(&mut self, frame: &mut Frame) -> () {
        match self {
            Window::Blank(ui) => ui.render(frame),
            Window::MainMenu(ui) => ui.render(frame),
            Window::NetCapSelection(ui) => ui.render(frame),
            Window::NetCapResult(ui) => ui.render(frame),
            Window::Progress(ui) => ui.render(frame),
        }
    }
    fn handle_event(&mut self, event: &Event) -> Action {
        match self {
            Window::Blank(ui) => ui.handle_event(event),
            Window::MainMenu(ui) => ui.handle_event(event),
            Window::NetCapSelection(ui) => ui.handle_event(event),
            Window::NetCapResult(ui) => ui.handle_event(event),
            Window::Progress(ui) => ui.handle_event(event),
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

pub struct ProgressUI {
    text: String,
    // progress: Arc<AtomicUsize>,
    status: Arc<Mutex<String>>,
}

impl ProgressUI {
    pub fn new(status: Arc<Mutex<String>>) -> Self {
        Self {
            text: String::from("Loading..."),
            status,
        }
    }
}

impl Render for ProgressUI {
    fn render(&mut self, frame: &mut Frame) -> () {
        frame.render_widget(
            Paragraph::new(self.text.clone())
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded))
                .alignment(Alignment::Center),
            frame.size(),
        )
    }

    fn handle_event(&mut self, event: &Event) -> Action {
        match event {
            Event::Tick => {
                self.text = self.status.lock().unwrap().clone();
                Action::None
            }
            Event::Key(_) => Action::Quit,
            Event::Mouse(_) => Action::None,
            Event::Resize(_, _) => Action::None,
        }
    }
}
