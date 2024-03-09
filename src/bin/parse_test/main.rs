use color_eyre::Result;

use dspf_parse::dspf::NomDspf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut file_path = "DSPF/nmos_trcp70.dspf";
    if args.len() > 1 {
        file_path = &args[1];
    }

    let _dspf = NomDspf::load(file_path);
    // dbg!(dspf);

    Ok(())
}
