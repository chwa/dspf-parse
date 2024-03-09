use core::fmt;
use std::collections::HashMap;

#[derive(Default)]
pub struct Netlist {
    pub all_nets: Vec<Net>,
    pub nets_map: HashMap<String, usize>,
    pub all_nodes: Vec<Node>,
    pub capacitors: Vec<Capacitor>,
}

impl Netlist {
    pub fn add_net(&mut self, net: Net) -> usize {
        self.nets_map.insert(net.info.name.clone(), self.all_nets.len());
        self.all_nets.push(net);
        self.all_nets.len() - 1
    }
    pub fn add_node(&mut self, node: Node) -> usize {
        self.all_nodes.push(node);
        self.all_nodes.len() - 1
    }
}

impl fmt::Debug for Netlist {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Netlist")
            .field("all_nets[truncated]", &&self.all_nets[..6])
            .field(
                "nets_map[truncated]",
                &self
                    .nets_map
                    .iter()
                    .take(6)
                    .map(|(s, n)| (s.as_str(), *n))
                    .collect::<Vec<(&str, usize)>>(),
            )
            .field("all_nodes[truncated]", &&self.all_nodes[..6])
            .finish()
    }
}

#[derive(Debug)]
pub enum NodeType {
    SubcktPin {
        pin_type: char,
        pin_cap: f64,
    },
    InstPin {
        inst_name: String,
        pin_name: String,
        pin_type: char,
        pin_cap: f64,
    },
    Ground,
    Other,
}

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub info: NodeType,
    pub coord: Option<(f64, f64)>,

    pub capacitors: Vec<usize>,
    pub of_net: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum NetType {
    GroundNode,
    SubcktPin,
    Other,
}

#[derive(Clone, Debug)]
pub struct NetInfo {
    pub name: String,
    pub net_type: NetType,
}

#[derive(Debug)]
pub struct Net {
    pub info: NetInfo,
    pub total_capacitance: f64,
    pub sub_nets: Vec<usize>,
    pub resistors: Vec<Resistor>,
}

#[derive(Debug)]
pub struct Resistor {
    pub nodes: (usize, usize),
    pub value: f64,
    pub layer: u8,
}

#[derive(Debug)]
pub struct Capacitor {
    pub nodes: (usize, usize),
    pub value: f64,
    pub layers: LayerInfo,
}

#[derive(Debug)]
pub enum LayerInfo {
    Single(u8),
    Pair(u8, u8),
    None,
}
