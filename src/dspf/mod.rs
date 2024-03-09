mod cont;
mod nomutil;

// pub mod netlist;
// mod pestdspf;
// pub use pestdspf::Dspf;

pub mod netlist2;
pub use netlist2 as netlist;
mod nomdspf2;
pub use nomdspf2::Dspf;

/// Load progress to be shared with another thread through Arc<Mutex>
#[derive(Default)]
pub struct LoadStatus {
    pub total_lines: usize,
    pub loaded_lines: usize,
    pub total_nets: usize,
    pub loaded_nets: usize,
    pub total_inst_blocks: usize,
    pub loaded_inst_blocks: usize,
}
