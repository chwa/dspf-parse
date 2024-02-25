use std::collections::HashMap;

// #[derive(Default)]
pub struct Netlist {
    pub all_nets: Vec<Net>,
    pub nets_map: HashMap<String, usize>,

    pub all_nodes: Vec<Node>,
    nodes_map: HashMap<String, usize>,

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

impl Netlist {
    pub fn new() -> Self {
        Netlist {
            all_nets: Vec::new(),
            nets_map: HashMap::new(),
            all_nodes: Vec::new(),
            nodes_map: HashMap::new(),
            all_parasitics: Vec::new(),
        }
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

    pub fn add_subnode(&mut self, subnode_name: &str, of_net: usize) {
        let node = Node {
            name: subnode_name.to_owned(),
            parasitics: vec![],
        };

        self.all_nodes.push(node);
        let index = self.all_nodes.len() - 1;

        self.nodes_map.insert(subnode_name.to_owned(), index);
        self.all_nets.get_mut(of_net).unwrap().sub_nets.push(index);
    }

    pub fn add_parasitic(
        &mut self,
        kind: &Primitive,
        node_a: &str,
        node_b: &str,
        value: f64,
    ) {
        let idx_a = self.nodes_map.get(node_a).unwrap();
        let idx_b = self.nodes_map.get(node_b).unwrap();

        let element = match kind {
            Primitive::R => Parasitic::R(*idx_a, *idx_b, value),
            Primitive::C => Parasitic::C(*idx_a, *idx_b, value),
        };

        self.all_parasitics.push(element);
        let index = self.all_parasitics.len() - 1;
        self.all_nodes
            .get_mut(*idx_a)
            .unwrap()
            .parasitics
            .push(index);
        self.all_nodes
            .get_mut(*idx_b)
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
    pub name: String,
    pub parasitics: Vec<usize>,
}

#[derive(Debug)]
pub enum Parasitic {
    R(usize, usize, f64),
    C(usize, usize, f64),
}
