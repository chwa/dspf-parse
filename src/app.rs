use crate::{
    dspf::Dspf,
    tui::Tui,
    uis::{main_menu::MainMenuUI, net_cap::NetCapUI, Render, SimpleUI, UI},
};
use color_eyre::Result;

pub enum Action {
    SelectionChanged(usize),
    SelectMenuOption(usize),
    SelectNet(usize),
    Quit,
    None,
}

pub struct App {
    pub tui: Tui,
    pub should_quit: bool,
    pub counter: u8,
    pub dspf: Option<Dspf>,
    pub dspf_path: Option<String>,
    current_ui: UI,
}

impl App {
    pub fn new() -> Result<Self> {
        let tui = Tui::new()?;
        Ok(Self {
            tui,
            should_quit: false,
            counter: 0,
            dspf: None,
            dspf_path: None,
            current_ui: UI::blank(),
        })
    }

    pub fn from_file_path(path: &str) -> Result<Self> {
        let mut app = Self::new()?;
        app.current_ui = UI::Simple(SimpleUI {
            text: String::from("Loading..."),
        });

        app.init()?;

        // hack to draw the loading screen before the main loop
        app.tui.draw(&mut app.current_ui)?;

        app.load_file(path);
        let dspf = &app.dspf.as_ref().unwrap();
        app.current_ui = UI::MainMenu(MainMenuUI::new(path, dspf));

        app.main_loop()?;
        app.cleanup()?;

        Ok(app)
    }

    pub fn load_file(&mut self, path: &str) {
        self.dspf = Some(Dspf::load(path));
        self.dspf_path = Some(path.to_owned());
    }

    pub fn init(&mut self) -> Result<()> {
        self.tui.enter()?;

        Ok(())
    }

    pub fn main_loop(&mut self) -> Result<()> {
        while !self.should_quit {
            self.tui.draw(&mut self.current_ui)?;

            let action = self.current_ui.handle_event(&self.tui.events.next()?);
            if let Action::Quit = action {
                self.quit()
            };
            match self.current_ui {
                UI::MainMenu(_) => match action {
                    Action::SelectMenuOption(i) => self.main_menu(i),
                    _ => {}
                },
                UI::NetCap(_) => match action {
                    Action::SelectMenuOption(i) => self.main_menu(i),
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn main_menu(&mut self, selection: usize) {
        if selection == 0 {
            self.current_ui = UI::NetCap(NetCapUI::new(
                self.dspf_path.as_ref().unwrap(),
                self.dspf.as_ref().unwrap(),
            ));
        }
    }

    pub fn cleanup(&mut self) -> Result<()> {
        self.tui.exit()
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
