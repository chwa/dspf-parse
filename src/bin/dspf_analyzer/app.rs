use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use dspf_parse::dspf::Dspf;

use color_eyre::Result;

use crate::{
    tui::Tui,
    windows::{
        main_menu::MainMenuUI, net_cap_result::NetCapResultUI,
        net_cap_selection::NetCapSelectionUI, ProgressUI, Render, Window,
    },
};

pub fn eng_format(value: f64) -> String {
    let map: [(i32, char); 10] = [
        (-18, 'a'),
        (-15, 'f'),
        (-12, 'p'),
        (-9, 'n'),
        (-6, 'u'),
        (-3, 'm'),
        (0, ' '),
        (3, 'k'),
        (6, 'M'),
        (9, 'G'),
    ];
    let mut log = value.abs().log10();
    if log.is_infinite() {
        log = 0.0;
    }

    let option = map
        .into_iter()
        .find(|(exp, _)| (*exp as f64) > log - 3.0)
        .unwrap_or((0, ' '));
    let mant = value / 10.0_f64.powf(option.0 as f64);
    let log_int = log.floor() as i32;
    let suffix = option.1;

    format!(
        "{mant:.prec$} {suffix}F",
        prec = (3 + option.0 - log_int) as usize
    )

    // match value {
    // 0..1e-15 =>

    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eng_format() {
        assert_eq!(&eng_format(0.0), "0.000  F");
        assert_eq!(&eng_format(1.0001), "1.000  F");
        assert_eq!(&eng_format(0.9999), "999.9 mF");
        assert_eq!(&eng_format(-0.9999), "-999.9 mF");
        assert_eq!(&eng_format(123.98), "124.0  F");
        assert_eq!(&eng_format(-123.98), "-124.0  F");
        assert_eq!(&eng_format(888.06e-15), "888.1 fF");
        assert_eq!(&eng_format(-888.06e-15), "-888.1 fF");
        assert_eq!(&eng_format(0.2388e9), "238.8 MF");
        assert_eq!(&eng_format(-0.2388e9), "-238.8 MF");
    }
}
pub enum Action {
    // SelectionChanged(usize),
    SelectMenuOption(usize),
    SelectNet(String),
    Quit,
    None,
}

pub struct App {
    pub tui: Tui,
    pub should_quit: bool,
    pub dspf: Option<Dspf>,
    pub dspf_path: Option<String>,
    current_ui: Window,
    pub joinhandle: Option<JoinHandle<Dspf>>,
}

impl App {
    pub fn new() -> Result<Self> {
        let tui = Tui::new()?;
        Ok(Self {
            tui,
            should_quit: false,
            dspf: None,
            dspf_path: None,
            current_ui: Window::blank(),
            joinhandle: None,
        })
    }

    pub fn from_file_path(path: &str) -> Result<Self> {
        let mut app = Self::new()?;

        let status: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        app.current_ui = Window::Progress(ProgressUI::new(Arc::clone(&status)));

        app.dspf_path = Some(path.to_owned());
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
                self.current_ui = Window::MainMenu(MainMenuUI::new(
                    self.dspf_path.as_ref().unwrap(),
                    &self.dspf.as_ref().unwrap(),
                ));
            }
        }
    }

    pub fn main_loop(&mut self) -> Result<()> {
        while !self.should_quit {
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
                self.dspf_path.as_ref().unwrap(),
                self.dspf.as_ref().unwrap(),
            ));
        } else if selection == 3 {
            self.should_quit = true;
        }
    }

    pub fn cleanup(&mut self) -> Result<()> {
        self.tui.exit()
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
