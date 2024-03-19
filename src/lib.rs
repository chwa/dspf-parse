pub mod dspf;

#[cfg(test)]
mod tests {
    use color_eyre::Result;

    #[test]
    fn load_dspf() -> Result<()> {
        let file_path = "DSPF/nmos_trcp70.dspf";

        let dspf = super::dspf::Dspf::load(file_path, None)?;
        dbg!(&dspf.netlist);

        assert_eq!(dspf.netlist.all_nets.len(), 12);

        Ok(())
    }
}
