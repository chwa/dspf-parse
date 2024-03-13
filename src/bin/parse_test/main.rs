use color_eyre::Result;

use dspf_parse::dspf::{netlist::AggrNet, Dspf};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut file_path = "DSPF/nmos_trcp70.dspf";
    if args.len() > 1 {
        file_path = &args[1];
    }

    let dspf = Dspf::load(file_path, None);

    let nl = dspf.netlist.unwrap();

    dbg!(nl.get_net_capacitors("out").unwrap());

    dbg!(nl.get_layer_capacitors("out", AggrNet::Total).unwrap());

    dbg!(nl.get_layer_capacitors("out", AggrNet::Net(String::from("ngate"))).unwrap());

    // dbg!(dspf);

    Ok(())
}
