mod nomutil;

pub mod netlist;
mod nomdspf;
pub use nomdspf::Dspf;

/// Load progress to be shared with another thread through Arc<Mutex>
#[derive(Default)]
pub struct LoadStatus {
    pub total_bytes: usize,
    pub loaded_bytes: usize,

    pub total_lines: usize,
    pub loaded_lines: usize,
    pub total_nets: usize,
    pub loaded_nets: usize,
    pub total_inst_blocks: usize,
    pub loaded_inst_blocks: usize,
}
