pub mod layer_cap_result;
pub mod main_menu;
pub mod net_cap_main;
pub mod net_cap_result;
pub mod net_cap_selection;

use std::sync::{Arc, Mutex};

use dspf_parse::dspf::LoadStatus;
use ratatui::Frame;
use ratatui::{layout::Alignment, text::Line};
use ratatui::{prelude::*, widgets::*};

use crate::{app::Action, event::Event};

use self::main_menu::MainMenuUI;
use self::net_cap_main::NetCapMainUI;

pub trait Render {
    fn render(&mut self, frame: &mut Frame) -> ();
    fn handle_event(&mut self, event: &Event) -> Action;
}

pub enum Window {
    Blank(BlankUI),
    MainMenu(MainMenuUI),
    NetCap(NetCapMainUI),
    Progress(ProgressUI),
}
use Window as W;

impl<'a> Default for Window {
    fn default() -> Self {
        W::Blank(BlankUI {})
    }
}

impl<'a> Render for Window {
    fn render(&mut self, frame: &mut Frame) -> () {
        match self {
            W::Blank(ui) => ui.render(frame),
            W::MainMenu(ui) => ui.render(frame),
            W::NetCap(ui) => ui.render(frame),
            W::Progress(ui) => ui.render(frame),
        }
    }
    fn handle_event(&mut self, event: &Event) -> Action {
        match self {
            W::Blank(ui) => ui.handle_event(event),
            W::MainMenu(ui) => ui.handle_event(event),
            W::NetCap(ui) => ui.handle_event(event),
            W::Progress(ui) => ui.handle_event(event),
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
    status: Arc<Mutex<LoadStatus>>,
}

impl ProgressUI {
    pub fn new(status: Arc<Mutex<LoadStatus>>) -> Self {
        Self { status }
    }
}

impl Render for ProgressUI {
    fn render(&mut self, frame: &mut Frame) -> () {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(10),
                Constraint::Fill(1),
            ])
            .split(frame.size());
        let cols_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(60),
                Constraint::Fill(1),
            ])
            .split(rows_layout[1]);

        let block = Block::new().borders(Borders::ALL).border_type(BorderType::Rounded);

        let inner_area = block.inner(cols_layout[1]);
        let rows_layout_inner = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Fill(4),
            ])
            .split(inner_area);

        let st = self.status.lock().unwrap();

        let lines = vec![
            Line::from(format!("{}/{} Bytes", st.loaded_bytes, st.total_bytes)),
            Line::from(format!(
                "{}/{} Instance blocks",
                st.loaded_inst_blocks, st.total_inst_blocks
            )),
        ];
        frame.render_widget(block, cols_layout[1]);

        frame.render_widget(
            Paragraph::new("Loading...").alignment(Alignment::Center),
            rows_layout_inner[0],
        );
        let mut ratio = (st.loaded_bytes as f64) / (st.total_bytes as f64);
        if ratio.is_nan() {
            ratio = 0.0;
        }

        frame.render_widget(
            Gauge::default()
                .use_unicode(true)
                .block(Block::bordered())
                .gauge_style(Color::Rgb(100, 100, 100))
                .ratio(ratio),
            rows_layout_inner[1],
        );

        frame.render_widget(
            Paragraph::new(lines).alignment(Alignment::Center),
            rows_layout_inner[2],
        );
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
