use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use dspf_parse::dspf::Dspf;
use dspf_parse::dspf::LoadStatus;

use color_eyre::Result;

use crate::{
    tui::Tui,
    windows::{
        main_menu::MainMenuUI, net_cap_result::NetCapResultUI,
        net_cap_selection::NetCapSelectionUI, ProgressUI, Render, Window,
    },
};

pub(crate) enum Action {
    SelectMenuOption(usize),
    SelectNet(String),
    Quit,
    None,
}

pub struct App {
    pub tui: Tui,
    pub running: bool,
    pub dspf: Option<Dspf>,
    current_ui: Window,
    pub joinhandle: Option<JoinHandle<Dspf>>,
}

impl App {
    pub fn new() -> Result<Self> {
        let tui = Tui::new()?;
        Ok(Self {
            tui,
            running: true,
            dspf: None,
            current_ui: Window::blank(),
            joinhandle: None,
        })
    }

    pub fn from_file_path(path: &str) -> Result<Self> {
        let mut app = Self::new()?;

        let status: Arc<Mutex<LoadStatus>> = Arc::new(Mutex::new(LoadStatus::default()));
        app.current_ui = Window::Progress(ProgressUI::new(Arc::clone(&status)));

        let p = path.to_owned();
        app.joinhandle = Some(thread::spawn(move || {
            Dspf::load(&p, Some(Arc::clone(&status)))
        }));

        app.init()?;
        app.main_loop()?;
        app.cleanup()?;

        Ok(app)
    }

    pub fn init(&mut self) -> Result<()> {
        self.tui.enter()?;

        Ok(())
    }

    fn try_join_loader(&mut self) {
        if let Some(j) = self.joinhandle.as_ref() {
            if j.is_finished() {
                let j = self.joinhandle.take().unwrap();
                let dspf = j.join().unwrap();
                self.dspf = Some(dspf);
                self.current_ui =
                    Window::MainMenu(MainMenuUI::new(&self.dspf.as_ref().unwrap()));
            }
        }
    }

    pub fn main_loop(&mut self) -> Result<()> {
        while self.running {
            self.try_join_loader();

            self.tui.draw(&mut self.current_ui)?;

            let action = self.current_ui.handle_event(&self.tui.events.next()?);

            if let Action::Quit = action {
                self.quit()
            };
            match &mut self.current_ui {
                Window::MainMenu(_) => match action {
                    Action::SelectMenuOption(i) => self.main_menu(i),
                    _ => {}
                },
                Window::NetCapSelection(ui) => match action {
                    Action::SelectNet(net_name) => {
                        let report = self
                            .dspf
                            .as_ref()
                            .unwrap()
                            .netlist
                            .as_ref()
                            .unwrap()
                            .get_net_capacitors(&net_name)
                            .unwrap();
                        ui.result_ui = NetCapResultUI::new(report);
                        // self.current_ui = Window::NetCapResult(NetCapResultUI::new(report));
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn main_menu(&mut self, selection: usize) {
        if selection == 0 {
            self.current_ui = Window::NetCapSelection(NetCapSelectionUI::new(
                self.dspf.as_ref().unwrap(),
            ));
        } else if selection == 3 {
            self.quit();
        }
    }

    pub fn cleanup(&mut self) -> Result<()> {
        self.tui.exit()
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
