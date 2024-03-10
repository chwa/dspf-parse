pub mod dspf;

#[cfg(test)]
mod tests {

    #[test]
    fn load_dspf() {
        let file_path = "DSPF/nmos_trcp70.dspf";

        let dspf = super::dspf::Dspf::load(file_path, None);
        assert!(dspf.netlist.is_some());

        assert_eq!(dspf.netlist.unwrap().all_nets.len(), 12);
    }
}
