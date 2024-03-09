use core::fmt;
use std::{cmp::min, collections::HashMap};

use color_eyre::{eyre::ContextCompat, Result};

#[derive(Default)]
pub struct Netlist {
    pub all_nets: Vec<Net>,
    pub nets_map: HashMap<String, usize>,
    pub all_nodes: Vec<Node>,
    pub capacitors: Vec<Capacitor>,
    pub layer_map: HashMap<u8, String>,
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
            .field("all_nets[truncated]", &&self.all_nets[..5])
            .field(
                "nets_map[truncated]",
                &self
                    .nets_map
                    .iter()
                    .take(5)
                    .map(|(s, n)| (s.as_str(), *n))
                    .collect::<Vec<(&str, usize)>>(),
            )
            .field("all_nodes[truncated]", &&self.all_nodes[..5])
            .field("capacitors[truncated]", &&self.capacitors[..5])
            .finish()
    }
}

#[derive(Debug)]
pub struct NetCapForAggressor {
    pub aggressor_name: String,
    pub cap: f64,
}
#[derive(Debug)]
pub struct NetCapForLayer {
    pub layer_names: (String, String),
    pub cap: f64,
}

#[derive(Default, Debug)]
pub struct NetCapReport {
    pub net_name: String,
    pub total_cap: f64,
    pub table: Vec<NetCapForAggressor>,
    // pub per_layer: Vec<NetCapForLayer>,
    // pub per_aggressor_per_layer: Vec<Vec<NetCapForLayer>>,
}

#[derive(Default, Debug)]
pub struct LayerCapReport {
    pub net_name: String,
    pub aggressor_name: Option<String>,
    pub total_cap: f64,
    pub table: Vec<NetCapForLayer>,
}

impl Netlist {
    pub fn get_net(&self, net: &str) -> Result<&Net> {
        let idx = self.nets_map.get(net).context("Net name not found")?;
        Ok(&self.all_nets[*idx])
    }

    pub fn get_net_capacitors(&self, net_name: &str) -> Result<NetCapReport> {
        let idx = self.nets_map.get(net_name).context("Net name not found")?;
        let net = &self.all_nets[*idx];

        let mut net_caps: HashMap<usize, f64> = HashMap::new();

        for subnode_idx in net.sub_nets.iter() {
            let subnode = &self.all_nodes[*subnode_idx];
            for cap in subnode.capacitors.iter().map(|s| &self.capacitors[*s]) {
                let other_node: usize;
                if cap.nodes.0 == *subnode_idx {
                    other_node = cap.nodes.1;
                } else {
                    other_node = cap.nodes.0;
                }

                let other_net = self.all_nodes[other_node].of_net;
                *net_caps.entry(other_net).or_insert(0.0) += cap.value;
            }
        }

        let mut per_aggressor: Vec<NetCapForAggressor> = Vec::new();
        for (idx, value) in net_caps.drain() {
            per_aggressor.push(NetCapForAggressor {
                aggressor_name: self.all_nets[idx].info.name.to_owned(),
                cap: value,
            });
        }
        per_aggressor.sort_by(|a, b| {
            b.cap.partial_cmp(&a.cap).unwrap().then(a.aggressor_name.cmp(&b.aggressor_name))
        });

        let report = NetCapReport {
            net_name: net_name.to_owned(),
            total_cap: net.total_capacitance,
            table: per_aggressor,
        };
        Ok(report)
    }

    pub fn get_layer_capacitors(
        &self,
        net_name: &str,
        aggressor_name: Option<&str>,
    ) -> Result<LayerCapReport> {
        let idx_self = self.nets_map.get(net_name).context("Net name not found")?;

        let net_self = &self.all_nets[*idx_self];
        // let net_other = &self.all_nets[*idx_other];

        let mut layer_caps: HashMap<(u8, u8), f64> = HashMap::new();
        let mut total_capacitance: f64 = 0.0;

        for subnode_idx in net_self.sub_nets.iter() {
            let subnode = &self.all_nodes[*subnode_idx];
            for cap in subnode.capacitors.iter().map(|s| &self.capacitors[*s]) {
                let mut layers: (u8, u8); // (our_layer, aggressor_layer)
                match cap.layers {
                    LayerInfo::Single(n1) => {
                        layers = (n1, 0);
                    }
                    LayerInfo::Pair(n1, n2) => {
                        layers = (n1, n2);
                    }
                    LayerInfo::None => {
                        layers = (0, 0);
                    }
                }
                let other_node: usize;
                if cap.nodes.0 == *subnode_idx {
                    other_node = cap.nodes.1;
                } else {
                    other_node = cap.nodes.0;
                    // TODO: assuming lvl1, lvl2 are the layers for node_a, node_b in the instance statement
                    layers = (layers.1, layers.0);
                }

                let other_net = self.all_nodes[other_node].of_net;
                if let Some(name) = aggressor_name {
                    let idx_aggressor = *self.nets_map.get(name).context("Net name not found")?;
                    if other_net != idx_aggressor {
                        continue;
                    }
                }
                *layer_caps.entry(layers).or_insert(0.0) += cap.value;
                total_capacitance += cap.value;
            }
        }

        let mut per_layer: Vec<NetCapForLayer> = Vec::new();
        for (idx, value) in layer_caps.drain() {
            per_layer.push(NetCapForLayer {
                layer_names: (
                    self.layer_map[&idx.0].to_owned(),
                    self.layer_map[&idx.1].to_owned(),
                ),
                cap: value,
            });
        }
        per_layer.sort_by(|a, b| b.cap.partial_cmp(&a.cap).unwrap());

        let report = LayerCapReport {
            net_name: net_name.to_owned(),
            aggressor_name: aggressor_name.map(|s| s.to_owned()),
            total_cap: total_capacitance,
            table: per_layer,
        };
        Ok(report)
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

pub struct Node {
    pub name: String,
    pub info: NodeType,
    pub coord: Option<(f64, f64)>,

    pub capacitors: Vec<usize>,
    pub of_net: usize,
}
impl fmt::Debug for Node {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Node")
            .field("name", &&self.name)
            .field("info", &&self.info)
            .field("coord", &&self.coord)
            .field(
                "capacitors[truncated]",
                &&self.capacitors[..min(5, self.capacitors.len())],
            )
            .field("of_net", &&self.of_net)
            .finish()
    }
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

pub struct Net {
    pub info: NetInfo,
    pub total_capacitance: f64,
    pub sub_nets: Vec<usize>,
    pub resistors: Vec<Resistor>,
}
impl fmt::Debug for Net {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Net")
            .field("info", &&self.info)
            .field("total_capacitance", &&self.total_capacitance)
            .field(
                "sub_nets[truncated]",
                &&self.sub_nets[..min(5, self.sub_nets.len())],
            )
            .field(
                "resistors[truncated]",
                &&self.resistors[..min(5, self.resistors.len())],
            )
            .finish()
    }
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
