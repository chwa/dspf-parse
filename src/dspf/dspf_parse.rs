use std::fs;

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use super::netlist::{Netlist, Primitive};

#[derive(Parser)]
#[grammar = "dspf/dspf.pest"]
pub struct DspfParser;

#[derive(Default)]
pub struct Dspf {
    pub file_path: String,
    pub file_contents: String,
    pub netlist: Option<Netlist>,
}

impl Dspf {
    pub fn load(file_path: &str) -> Dspf {
        let file_contents = fs::read_to_string(file_path)
            .expect("cannot read file")
            .replace("\n+", "");

        let mut netlist: Option<Netlist> = None;

        let pairs = DspfParser::parse(Rule::file, &file_contents).ok();

        for item in pairs.unwrap() {
            match item.as_rule() {
                Rule::subckt => {
                    netlist = Some(load_subckt(item));
                }
                _ => {}
            };
        }

        Dspf {
            file_path: file_path.to_owned(),
            file_contents: file_contents,
            netlist: netlist,
        }
    }

    pub fn parse_netlist(self) {
        let netlist = self.netlist.unwrap();

        for net in netlist.all_nets.iter() {
            let cap_sum = netlist.cap_for_net(&net.name);

            println!(
                "Net {:16} C={:.3e} (sum={:.3e} over {} subnodes)",
                net.name,
                net.total_capacitance,
                cap_sum,
                net.sub_nets.len()
            );
        }

        println!("\nTotal {} subnodes", netlist.all_nodes.len());
        println!("Parasitics: {}", netlist.all_parasitics.len());
    }
}

// fn list_of_strings(pair: Pair<'_, Rule>) -> Vec<String> {
//     pair.into_inner().map(|x| x.as_str().to_owned()).collect()
// }

// fn list_of_key_value(pair: Pair<'_, Rule>) -> Vec<(String, String)> {
//     pair.into_inner()
//         .map(|x| {
//             let mut a = x.into_inner();
//             (
//                 a.next().unwrap().as_str().to_owned(),
//                 a.next().unwrap().as_str().to_owned(),
//             )
//         })
//         .collect()
// }

fn load_subckt(subckt_pair: Pair<'_, Rule>) -> Netlist {
    let parts = subckt_pair.into_inner();

    let mut netlist = Netlist::new();

    // TODO: ground node
    netlist.create_net("0", 0.0);
    netlist.add_subnode("0", 0);

    let mut parasitics: Vec<(Primitive, String, String, String, f64)> = Vec::new();

    for item in parts {
        match item.as_rule() {
            Rule::dspf_net_section => {
                load_net_section(item, &mut netlist);
            }
            Rule::primitive_stmt => {
                let mut item = item.into_inner();
                let name = item.next().unwrap().as_str();
                let node_a = item.next().unwrap().as_str();
                let node_b = item.next().unwrap().as_str();
                let value = item.next().unwrap().as_str().parse::<f64>().unwrap();

                let kind = match name.chars().next().unwrap() {
                    'R' => Some(Primitive::R),
                    'C' => Some(Primitive::C),
                    _ => None,
                };

                if let Some(kind) = kind {
                    parasitics.push((
                        kind,
                        name.to_owned(),
                        node_a.to_owned(),
                        node_b.to_owned(),
                        value,
                    ));
                }
            }
            Rule::instance_stmt => {
                // todo!()
            }
            _ => {}
        }
    }

    // at this point we've parsed all subnode statements, so it's safe to call add_parasitic()
    // (which looks up the nodes in the HashMap)
    for (kind, _, node_a, node_b, value) in &parasitics {
        netlist.add_parasitic(kind, node_a, node_b, *value);
    }

    netlist
}

fn load_net_section(net_pair: Pair<'_, Rule>, netlist: &mut Netlist) {
    let parts = net_pair.into_inner();
    let mut current_index: usize = 0;
    for item in parts {
        // dbg!(item.as_rule());
        match item.as_rule() {
            Rule::dspf_net_line => {
                let mut contents = item.into_inner();
                let net_name = contents.next().unwrap().as_str().to_owned();
                let capacitance =
                    contents.next().unwrap().as_str().parse::<f64>().unwrap();
                current_index = netlist.create_net(&net_name, capacitance);
            }
            Rule::dspf_pin_line => {
                let mut contents = item.into_inner();
                let subnode_name = contents.next().unwrap().as_str().to_owned();
                netlist.add_subnode(&subnode_name, current_index);
            }
            Rule::dspf_inst_line => {
                let mut contents = item.into_inner();
                let subnode_name = contents.next().unwrap().as_str().to_owned();
                // dbg!(&subnode_name);
                netlist.add_subnode(&subnode_name, current_index)
            }
            Rule::dspf_subnode_line => {
                let mut contents = item.into_inner();
                let subnode_name = contents.next().unwrap().as_str().to_owned();
                // println!("{}", subnode_name);
                netlist.add_subnode(&subnode_name, current_index)
            }
            _ => {}
        }
    }
}

// #[derive(Debug, Default)]
// struct Inst {
//     name: String,
//     pins: Vec<String>,
//     model: String,
//     params: Vec<(String, String)>,
// }

// impl Inst {
//     fn parse(inst_stmt: Pair<'_, Rule>) -> Self {
//         let mut parts = inst_stmt.into_inner();

//         Self {
//             name: parts.next().unwrap().as_str().to_owned(),
//             pins: list_of_strings(parts.next().unwrap()),
//             model: parts.next().unwrap().as_str().to_owned(),
//             params: list_of_key_value(parts.next().unwrap()),
//         }
//     }
// }
