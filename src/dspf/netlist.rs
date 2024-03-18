use core::fmt;
use std::{cmp::min, collections::HashMap, fmt::Formatter};

use color_eyre::{
    eyre::{Context, ContextCompat, OptionExt},
    Result,
};
use faer::{solvers::SpSolver, sparse::SparseColMat, Col, Side};

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

    pub fn get_net(&self, net: &str) -> Result<&Net> {
        let idx = self.nets_map.get(net).context("Net name not found")?;
        Ok(&self.all_nets[*idx])
    }

    pub fn get_net_capacitors(&self, net_name: &str) -> Result<NetCapReport> {
        let idx = self.nets_map.get(net_name).context("Net name not found")?;
        let net = &self.all_nets[*idx];

        let mut net_caps: HashMap<usize, f64> = HashMap::new();

        for subnode_idx in net.subnodes.iter() {
            let subnode = &self.all_nodes[*subnode_idx];
            for cap in subnode.capacitors.iter().map(|s| &self.capacitors[*s]) {
                let other_node: usize = if cap.nodes.0 == *subnode_idx {
                    cap.nodes.1
                } else {
                    cap.nodes.0
                };

                let other_net = self.all_nodes[other_node].of_net;
                *net_caps.entry(other_net).or_insert(0.0) += cap.value;
            }
        }

        let mut per_aggressor: Vec<NetCapForAggressor> = Vec::new();
        for (idx, value) in net_caps.drain() {
            per_aggressor.push(NetCapForAggressor {
                aggressor: AggrNet::Net(self.all_nets[idx].info.name.to_owned()),
                cap: value,
            });
        }
        per_aggressor
            .sort_by(|a, b| b.cap.partial_cmp(&a.cap).unwrap().then(a.aggressor.cmp(&b.aggressor)));

        let mut total_cap = net.total_capacitance;

        if total_cap.is_nan() {
            total_cap = per_aggressor.iter().map(|x| x.cap).sum()
        }

        let report = NetCapReport {
            net_name: net_name.to_owned(),
            total_cap: NetCapForAggressor {
                aggressor: AggrNet::Total,
                cap: total_cap,
            },
            table: per_aggressor,
        };
        Ok(report)
    }

    pub fn get_layer_capacitors(
        &self,
        net_name: &str,
        aggressor_net: AggrNet,
    ) -> Result<LayerCapReport> {
        let idx_self = self.nets_map.get(net_name).context("Net name not found")?;

        let net_self = &self.all_nets[*idx_self];
        // let net_other = &self.all_nets[*idx_other];

        let mut layer_caps: HashMap<(u8, u8), f64> = HashMap::new();
        let mut total_capacitance: f64 = 0.0;

        for subnode_idx in net_self.subnodes.iter() {
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
                if let AggrNet::Net(ref name) = aggressor_net {
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
            aggressor_net,
            total_cap: total_capacitance,
            table: per_layer,
        };
        Ok(report)
    }

    pub fn get_path_resistance(
        &self,
        net_name: &str,
        input_names: &[String],
        output_names: &[String],
    ) -> Result<ResReport> {
        let net = self.get_net(net_name).wrap_err("Net not found.")?;

        let mut input_nodes_in_order: Vec<usize> = Vec::new();
        for node_name in input_names {
            let n = net
                .subnodes
                .iter()
                .find(|&&node_idx| self.all_nodes[node_idx].name == *node_name)
                .ok_or_eyre(format!("Input node not found: {}.", node_name))?;
            input_nodes_in_order.push(*n);
        }

        // Check that output nodes exist and collect node indices in the order that the caller requested
        let mut output_nodes_in_order: Vec<usize> = Vec::new();
        for node_name in output_names {
            let n = net
                .subnodes
                .iter()
                .find(|&&node_idx| self.all_nodes[node_idx].name == *node_name)
                .ok_or_eyre(format!("Output node not found: {}.", node_name))?;
            output_nodes_in_order.push(*n);
        }

        let (mut output_nodes, mut other_nodes): (Vec<_>, Vec<_>) = net
            .subnodes
            .iter()
            .copied()
            .filter(|node| !input_names.contains(&self.all_nodes[*node].name))
            .partition(|node| output_names.contains(&self.all_nodes[*node].name));

        let num_outputs = output_nodes.len();
        output_nodes.sort();
        other_nodes.sort();

        let mut nodes = output_nodes;
        nodes.append(&mut other_nodes);

        let mut entries: Vec<(usize, usize, f64)> = Vec::new();
        let mut conductance: Vec<f64> = Vec::new();

        for (row, res) in net.resistors.iter().enumerate() {
            // need to search in the 2 sorted partitions (outputs and others)
            if let Ok(col) = nodes[..num_outputs].binary_search(&res.nodes.0).or_else(|_| {
                nodes[num_outputs..].binary_search(&res.nodes.0).map(|idx| idx + num_outputs)
            }) {
                entries.push((row, col, 1.0));
            }
            if let Ok(col) = nodes[..num_outputs].binary_search(&res.nodes.1).or_else(|_| {
                nodes[num_outputs..].binary_search(&res.nodes.1).map(|idx| idx + num_outputs)
            }) {
                entries.push((row, col, -1.0));
            }

            conductance.push(1.0 / res.value);
        }

        let incidence =
            SparseColMat::try_new_from_triplets(net.resistors.len(), nodes.len(), &entries)
                .unwrap();

        let cond_triplets: Vec<_> =
            conductance.iter().enumerate().map(|(i, &g)| (i, i, g)).collect();

        let conductance = SparseColMat::try_new_from_triplets(
            cond_triplets.len(),
            cond_triplets.len(),
            cond_triplets.as_slice(),
        )
        .unwrap();

        let x = &conductance * &incidence; // cond.len() x nodes.len()
        let g_matrix = incidence.to_owned().unwrap().into_transpose().to_col_major().unwrap() * x;

        let mut b: Col<f64> = Col::zeros(nodes.len());

        for i in 0..num_outputs {
            b[i] = 1.0 / num_outputs as f64;
        }

        let llt = g_matrix.sp_cholesky(Side::Lower).unwrap();

        let voltages = llt.solve(b);

        let mut voltages_in_order: Vec<f64> = Vec::new();
        for n in output_nodes_in_order {
            voltages_in_order.push(voltages[nodes[..num_outputs].binary_search(&n).unwrap()]);
        }
        let total_res =
            voltages_in_order.iter().fold(0.0, |acc, x| acc + x) / voltages_in_order.len() as f64;

        let v_res = incidence * voltages;

        let power = v_res.to_owned().column_vector_into_diagonal() * (conductance * v_res);

        let mut power_per_layer: HashMap<u8, f64> = HashMap::new();

        for (res, value) in net.resistors.iter().zip(power.as_slice().iter()) {
            *power_per_layer.entry(res.layer).or_insert(0.0) += value;
        }

        let table_layers: Vec<_> = power_per_layer
            .iter()
            .map(|(i, value)| ResForLayer {
                layer_name: self.layer_map[i].clone(),
                res: *value,
            })
            .collect();

        Ok(ResReport {
            net_name: net_name.to_owned(),
            input_nodes: input_names.to_vec(),
            total_res,
            table_outputs: output_names
                .iter()
                .zip(voltages_in_order.iter())
                .map(|(n, v)| NodeResistance {
                    node: n.clone(),
                    resistance: *v,
                })
                .collect(),
            table_layers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_netlist_r() -> Result<()> {
        let mut nl = Netlist::default();

        let mut net = Net {
            info: NetInfo {
                name: String::from("mynet"),
                net_type: NetType::Other,
            },
            total_capacitance: 1.2e-12,
            subnodes: Vec::new(),
            resistors: Vec::new(),
        };

        let node = Node {
            name: String::from("mynet"),
            info: NodeType::Other,
            coord: None,
            capacitors: vec![],
            of_net: 0,
        };
        net.subnodes.push(nl.add_node(node));

        let node = Node {
            name: String::from("node_1"),
            info: NodeType::Other,
            coord: None,
            capacitors: vec![],
            of_net: 0,
        };
        net.subnodes.push(nl.add_node(node));

        let node = Node {
            name: String::from("node_2"),
            info: NodeType::Other,
            coord: None,
            capacitors: vec![],
            of_net: 0,
        };
        net.subnodes.push(nl.add_node(node));

        let node = Node {
            name: String::from("node_3"),
            info: NodeType::Other,
            coord: None,
            capacitors: vec![],
            of_net: 0,
        };
        net.subnodes.push(nl.add_node(node));

        net.resistors.push(Resistor {
            nodes: (0, 1),
            value: 100.0,
            layer: 0,
        });
        net.resistors.push(Resistor {
            nodes: (1, 2),
            value: 200.0,
            layer: 0,
        });
        net.resistors.push(Resistor {
            nodes: (1, 3),
            value: 300.0,
            layer: 0,
        });

        nl.add_net(net);

        let inputs = vec![String::from("mynet")];
        let outputs = vec![String::from("node_3"), String::from("node_2")];
        nl.get_path_resistance("mynet", &inputs, &outputs)?;
        Ok(())
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

#[derive(Debug, Clone, Default)]
pub struct NetCapForAggressor {
    pub aggressor: AggrNet,
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
    pub total_cap: NetCapForAggressor,
    pub table: Vec<NetCapForAggressor>,
    // pub per_layer: Vec<NetCapForLayer>,
    // pub per_aggressor_per_layer: Vec<Vec<NetCapForLayer>>,
}

#[derive(Default, Debug)]
pub struct LayerCapReport {
    pub net_name: String,
    pub aggressor_net: AggrNet,
    pub total_cap: f64,
    pub table: Vec<NetCapForLayer>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AggrNet {
    #[default]
    Total,
    Net(String),
}

impl fmt::Display for AggrNet {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Total => write!(f, "[TOTAL]"),
            Self::Net(net_name) => write!(f, "{}", net_name),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct NodeResistance {
    pub node: String,
    pub resistance: f64,
}

#[derive(Debug, Clone, Default)]
pub struct ResForLayer {
    pub layer_name: String,
    pub res: f64,
}

#[derive(Default, Debug)]
pub struct ResReport {
    pub net_name: String,
    pub input_nodes: Vec<String>,
    pub total_res: f64,
    pub table_outputs: Vec<NodeResistance>,
    pub table_layers: Vec<ResForLayer>,
}

#[derive(Debug, PartialEq, PartialOrd)]
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
    pub subnodes: Vec<usize>,
    pub resistors: Vec<Resistor>,
}
impl fmt::Debug for Net {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Net")
            .field("info", &&self.info)
            .field("total_capacitance", &&self.total_capacitance)
            .field(
                "sub_nets[truncated]",
                &&self.subnodes[..min(5, self.subnodes.len())],
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
