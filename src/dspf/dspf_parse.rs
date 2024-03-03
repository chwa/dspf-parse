use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
    sync::{Arc, Mutex},
};

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use super::{
    cont::ContinuedLines,
    netlist::{Netlist, Primitive},
};

#[derive(Parser)]
#[grammar = "dspf/dspf.pest"]
pub struct DspfParser;

#[derive(Default)]
pub struct Dspf {
    pub file_path: String,
    pub file_size: u64,
    pub netlist: Option<Netlist>,
}

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

impl Dspf {
    pub fn load(file_path: &str, status: Option<Arc<Mutex<LoadStatus>>>) -> Dspf {
        let f = File::open(file_path).unwrap();
        let filesize = f.metadata().unwrap().len();
        let f = BufReader::new(f);

        let lines: Vec<(usize, String)> = ContinuedLines::from_buf(f)
            .collect::<io::Result<Vec<_>>>()
            .unwrap();

        let (_name, _pins, inner) = get_subckt(&lines);

        let (net_blocks, inst_blocks) = get_net_blocks(inner);

        let num_lines = net_blocks.iter().map(|b| b.len()).sum::<usize>()
            + inst_blocks.iter().map(|b| b.len()).sum::<usize>();
        let mut loaded_lines = 0_usize;

        if let Some(ref s) = status {
            let mut status = s.lock().unwrap();
            *status = LoadStatus {
                total_lines: num_lines,
                total_nets: net_blocks.len(),
                total_inst_blocks: inst_blocks.len(),
                ..LoadStatus::default()
            };
        }

        let mut netlist = Netlist::new();
        let mut nodes_map: HashMap<String, usize> = HashMap::new();
        // TODO: ground node
        netlist.create_net("0", 0.0);
        nodes_map.insert(String::from("0"), 0);

        for (i, block) in net_blocks.iter().enumerate() {
            let mut it = block.iter(); // TODO: into_iter???
            let line = it.next().unwrap();
            let mut pairs = DspfParser::parse(Rule::dspf_net_line, &line.1).unwrap();
            let net_name = pairs.next().unwrap().as_str();
            let net_cap = pairs.next().unwrap().as_str().parse::<f64>().unwrap();

            let net_id = netlist.create_net(&net_name, net_cap);
            // we add the net as a subnote here (most nets also appear in the *|S subnet definitions, but not all...)
            nodes_map.insert(net_name.to_owned(), net_id);

            for (_, line) in it {
                let element = DspfParser::parse(Rule::dspf_net_element, line)
                    .unwrap()
                    .next()
                    .unwrap();
                match element.as_rule() {
                    Rule::dspf_pin_line => {
                        let mut inner = element.into_inner();
                        let pin_name = inner.next().unwrap().as_str();
                        // TODO other pin params
                        let index = netlist.add_subnode(pin_name, net_id);
                        nodes_map.insert(pin_name.to_owned(), index);
                    }
                    Rule::dspf_inst_line => {
                        let mut inner = element.into_inner();
                        let node_name = inner.next().unwrap().as_str();
                        let _inst_name = inner.next().unwrap().as_str();
                        let _pin_name = inner.next().unwrap().as_str();
                        let index = netlist.add_subnode(node_name, net_id);
                        nodes_map.insert(node_name.to_owned(), index);
                    }
                    Rule::dspf_subnode_line => {
                        let mut inner = element.into_inner();
                        let node_name = inner.next().unwrap().as_str();
                        let index = netlist.add_subnode(node_name, net_id);
                        nodes_map.insert(node_name.to_owned(), index);
                    }
                    _ => {}
                }
                loaded_lines += 1;
                if let Some(ref s) = status {
                    let mut status = s.lock().unwrap();
                    status.loaded_nets = i + 1;
                    status.loaded_lines = loaded_lines;
                }
            }
        }
        for (i, block) in inst_blocks.iter().enumerate() {
            let it = block.iter();
            for (_, line) in it {
                let mut element = DspfParser::parse(Rule::primitive_stmt, line).unwrap();

                let inst_name = element.next().unwrap().as_str();
                let node_a = element.next().unwrap().as_str();
                let node_a: usize = *nodes_map.get(node_a).unwrap();
                let node_b = element.next().unwrap().as_str();
                let node_b: usize = *nodes_map.get(node_b).unwrap();
                let value = element.next().unwrap().as_str().parse::<f64>().unwrap();

                let kind = match inst_name.chars().next().unwrap() {
                    'R' => Some(Primitive::R),
                    'C' => Some(Primitive::C),
                    _ => None,
                };

                netlist.add_parasitic(&kind.unwrap(), node_a, node_b, value);
                loaded_lines += 1;
                if let Some(ref s) = status {
                    let mut status = s.lock().unwrap();
                    status.loaded_inst_blocks = i + 1;
                    status.loaded_lines = loaded_lines;
                }
            }
        }

        Dspf {
            file_path: file_path.to_owned(),
            file_size: filesize,
            netlist: Some(netlist),
        }
    }
}

fn list_of_strings(pair: Pair<'_, Rule>) -> Vec<String> {
    pair.into_inner().map(|x| x.as_str().to_owned()).collect()
}

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

fn get_subckt(lines: &[(usize, String)]) -> (String, Vec<String>, &[(usize, String)]) {
    let mut lines_iter = lines.iter();
    let subckt_start = lines_iter.position(|l| l.1.starts_with(".SUBCKT")).unwrap();
    let subckt_end = lines_iter.position(|l| l.1.starts_with(".ENDS")).unwrap();

    let mut pairs = DspfParser::parse(Rule::subckt_line, &lines[subckt_start].1).unwrap();
    let name = pairs.next().unwrap().as_str().to_owned();
    let pins = list_of_strings(pairs.next().unwrap());

    let inner = &lines[subckt_start + 1..subckt_end];
    (name, pins, inner)
}

type Block<'a> = &'a [(usize, String)];

fn get_net_blocks(lines: &[(usize, String)]) -> (Vec<Block>, Vec<Block>) {
    let mut net_blocks: Vec<&[(usize, String)]> = Vec::new();
    let mut instance_blocks: Vec<&[(usize, String)]> = Vec::new();
    let mut it = lines.iter().enumerate().peekable();
    while let Some((i, (_, text))) = it.next() {
        if text.starts_with("*|NET") {
            let net_start = i;
            let mut net_end = i;
            while let Some((i, _)) = it.next_if(|(_, (_, text))| {
                text.starts_with("*|") && !text.starts_with("*|NET")
            }) {
                net_end = i;
            }
            net_blocks.push(&lines[net_start..=net_end]);

            let inst_start = net_end + 1;
            let mut inst_end = inst_start;
            while let Some((i, _)) = it
                .next_if(|(_, (_, text))| text.starts_with("R") || text.starts_with("C"))
            {
                inst_end = i;
            }
            if inst_end != inst_start {
                instance_blocks.push(&lines[inst_start..=inst_end]);
            }
        }
    }

    (net_blocks, instance_blocks)
}
