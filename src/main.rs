pub mod app;
pub mod dspf;
pub mod event;
pub mod tui;
pub mod uis;

use color_eyre::Result;

use app::App;

// use crate::dspf::Dspf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut file_path = "DSPF/nmos_trcp70.dspf";
    // let mut file_path = "DSPF/dcdc_error_amp_trcp70.dspf";
    // let file_path = "DSPF/dcdc_ps_250mohm_trcp70.dspf";

    if args.len() > 1 {
        file_path = &args[1];
    }

    App::from_file_path(file_path)?;

    Ok(())
}
