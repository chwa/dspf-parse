use color_eyre::Result;

mod app;
mod event;
mod tui;
mod util;
mod windows;

use app::App;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut file_path = "DSPF/nmos_trcp70.dspf";
    if args.len() > 1 {
        file_path = &args[1];
    }

    App::from_file_path(file_path)?;

    Ok(())
}
