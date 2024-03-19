use color_eyre::{eyre::eyre, Result};

mod app;
mod event;
mod tui;
mod util;
mod windows;

use app::App;

fn main() -> Result<()> {
    // color_eyre::install()?;

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Err(eyre!("No DSPF filename provided."));
    }
    let file_path = &args[1];
    App::run(file_path)?;

    Ok(())
}
