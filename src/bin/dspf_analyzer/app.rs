use std::{
    fmt,
    rc::Rc,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use dspf_parse::dspf::LoadStatus;
use dspf_parse::dspf::{netlist::AggrNet, Dspf};

use color_eyre::Result;

use crate::{
    tui::Tui,
    windows::{
        main_menu::MainMenuUI, net_cap_main::NetCapMainUI, res_main::ResMainUI, ProgressUI, Render,
        Window,
    },
};

pub enum Action {
    SelectMenuOption(MainMenuOption),
    SelectNet(Option<String>),
    SelectAggrNet(Option<AggrNet>),
    SelectResNet(String),
    NodesChanged,
    MainMenu,
    Quit,
    None,
}

#[derive(Clone, Copy)]
pub enum MainMenuOption {
    CapAnalysis,
    ResAnalysis,
    Quit,
}

static MENU_OPTIONS: [MainMenuOption; 3] = [
    MainMenuOption::CapAnalysis,
    MainMenuOption::ResAnalysis,
    MainMenuOption::Quit,
];

impl fmt::Display for MainMenuOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MainMenuOption::CapAnalysis => write!(f, " Report capacitance for net..."),
            MainMenuOption::ResAnalysis => write!(f, " Path resistance [experimental]..."),
            MainMenuOption::Quit => write!(f, " Quit"),
        }
    }
}

pub struct App {
    pub tui: Tui,
    pub running: bool,
    pub dspf: Option<Rc<Dspf>>,
    current_ui: Window,
    pub joinhandle: Option<JoinHandle<Result<Dspf>>>,
}

impl App {
    pub fn new() -> Result<Self> {
        let tui = Tui::new()?;
        Ok(Self {
            tui,
            running: true,
            dspf: None,
            current_ui: Default::default(),
            joinhandle: None,
        })
    }

    pub fn run(path: &str) -> Result<()> {
        let mut app = Self::new()?;
        app.init()?;

        let status: Arc<Mutex<LoadStatus>> = Arc::new(Mutex::new(LoadStatus::default()));
        app.current_ui = Window::Progress(ProgressUI::new(Arc::clone(&status)));

        let p = path.to_owned();
        app.joinhandle = Some(thread::spawn(move || -> Result<Dspf> {
            Dspf::load(&p, Some(Arc::clone(&status)))
        }));

        let x = app.main_loop();
        app.cleanup()?;
        x
    }

    pub fn init(&mut self) -> Result<()> {
        self.tui.enter()?;

        Ok(())
    }

    fn try_join_loader(&mut self) -> Result<()> {
        if let Some(j) = self.joinhandle.as_ref() {
            if j.is_finished() {
                let j = self.joinhandle.take().unwrap();
                let dspf = j.join().unwrap()?; // propagate panic, return err if thread returned err

                let dspf = Rc::new(dspf);
                self.current_ui = Window::MainMenu(MainMenuUI::new(&dspf, &MENU_OPTIONS));
                self.dspf = Some(dspf);
            }
        }
        Ok(())
    }

    pub fn main_loop(&mut self) -> Result<()> {
        while self.running {
            self.try_join_loader()?;

            self.tui.draw(&mut self.current_ui)?;

            let action = self.current_ui.handle_event(&self.tui.events.next()?);

            match action {
                Action::Quit => self.quit(),
                Action::MainMenu => {
                    if let Some(dspf) = &self.dspf {
                        self.current_ui = Window::MainMenu(MainMenuUI::new(dspf, &MENU_OPTIONS));
                    }
                }
                Action::SelectMenuOption(option) => self.main_menu(option),
                _ => {}
            }
        }
        Ok(())
    }

    fn main_menu(&mut self, option: MainMenuOption) {
        if let Some(dspf) = &self.dspf {
            match option {
                MainMenuOption::CapAnalysis => {
                    self.current_ui = Window::NetCap(NetCapMainUI::new(dspf.clone()));
                }
                MainMenuOption::ResAnalysis => {
                    self.current_ui = Window::Res(ResMainUI::new(dspf.clone()));
                }
                MainMenuOption::Quit => {
                    self.quit();
                }
            }
        }
    }

    pub fn cleanup(&mut self) -> Result<()> {
        self.tui.exit()
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
