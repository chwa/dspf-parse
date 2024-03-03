use color_eyre::{eyre::ContextCompat, Result};
use std::collections::HashMap;

pub struct Netlist {
    pub all_nets: Vec<Net>,
    pub nets_map: HashMap<String, usize>,

    pub all_nodes: Vec<Node>,

    pub all_parasitics: Vec<Parasitic>,
}

impl std::fmt::Debug for Netlist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Netlist")
            .field("all_nets", &self.all_nets)
            .field("all_nodes", &self.all_nets)
            .finish()
    }
}

pub enum Primitive {
    C,
    R,
}

pub struct NetCapForAggressor {
    pub aggressor_name: String,
    pub cap: f64,
}
pub struct NetCapForLayer {
    pub layer_name: String,
    pub cap: f64,
}

#[derive(Default)]
pub struct NetCapReport {
    pub net_name: String,
    pub total_cap: f64,
    pub per_aggressor: Vec<NetCapForAggressor>,
    pub per_layer: Vec<NetCapForLayer>,
}

impl Netlist {
    pub fn new() -> Self {
        Netlist {
            all_nets: Vec::new(),
            nets_map: HashMap::new(),
            all_nodes: Vec::new(),
            all_parasitics: Vec::new(),
        }
    }

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
            for parasitic in subnode.parasitics.iter().map(|s| &self.all_parasitics[*s]) {
                if let Parasitic::C(node_a, node_b, value) = parasitic {
                    let other_node: usize;
                    if node_a == subnode_idx {
                        other_node = *node_b;
                    } else {
                        other_node = *node_a;
                    }
                    let other_net = self.all_nodes[other_node].of_net;
                    *net_caps.entry(other_net).or_insert(0.0) += value;
                }
            }
        }
        let mut per_aggressor: Vec<NetCapForAggressor> = Vec::new();
        let per_layer: Vec<NetCapForLayer> = Vec::new();
        for (idx, value) in net_caps.drain() {
            per_aggressor.push(NetCapForAggressor {
                aggressor_name: self.all_nets[idx].name.to_owned(),
                cap: value,
            });
        }
        per_aggressor.sort_by(|a, b| b.cap.partial_cmp(&a.cap).unwrap());
        let report = NetCapReport {
            net_name: net_name.to_owned(),
            total_cap: net.total_capacitance,
            per_aggressor: per_aggressor,
            per_layer: per_layer,
        };
        Ok(report)
    }

    pub fn create_net(&mut self, name: &str, capacitance: f64) -> usize {
        let net = Net {
            name: name.to_owned(),
            total_capacitance: capacitance,
            sub_nets: vec![],
        };
        self.all_nets.push(net);
        let index = self.all_nets.len() - 1;
        self.nets_map.insert(name.to_owned(), index);

        // we add the net as a subnote here (most nets also appear in the *|S subnet definitions, but not all...)
        self.add_subnode(name, index);
        index
    }

    pub fn add_subnode(&mut self, subnode_name: &str, of_net: usize) -> usize {
        let node = Node {
            parasitics: vec![],
            of_net: of_net,
        };

        self.all_nodes.push(node);
        let index = self.all_nodes.len() - 1;

        self.all_nets.get_mut(of_net).unwrap().sub_nets.push(index);
        index
    }

    pub fn add_parasitic(
        &mut self,
        kind: &Primitive,
        node_a: usize,
        node_b: usize,
        value: f64,
    ) {
        let element = match kind {
            Primitive::R => Parasitic::R(node_a, node_b, value),
            Primitive::C => Parasitic::C(node_a, node_b, value),
        };

        self.all_parasitics.push(element);
        let index = self.all_parasitics.len() - 1;
        self.all_nodes
            .get_mut(node_a)
            .unwrap()
            .parasitics
            .push(index);
        self.all_nodes
            .get_mut(node_b)
            .unwrap()
            .parasitics
            .push(index);
    }

    pub fn cap_for_net(&self, net_name: &str) -> f64 {
        let mut total = 0.0;
        let net_idx = self.nets_map.get(net_name).unwrap();
        for subnode_idx in self.all_nets[*net_idx].sub_nets.iter() {
            for p_idx in self.all_nodes[*subnode_idx].parasitics.iter() {
                if let Parasitic::C(_, _, value) = self.all_parasitics[*p_idx] {
                    total += value;
                }
            }
        }
        total
    }
}

pub struct Net {
    pub name: String,
    pub total_capacitance: f64,
    pub sub_nets: Vec<usize>,
    // instance_pins: ...
}

impl std::fmt::Debug for Net {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Net")
            .field("name", &self.name)
            .field("total_capacitance", &self.total_capacitance)
            .field("sub_nets", &Vec::from_iter(self.sub_nets.iter().take(5)))
            .finish()
    }
}

#[derive(Debug)]
pub struct Node {
    pub parasitics: Vec<usize>,
    pub of_net: usize,
}

#[derive(Debug)]
pub enum Parasitic {
    R(usize, usize, f64),
    C(usize, usize, f64),
}
