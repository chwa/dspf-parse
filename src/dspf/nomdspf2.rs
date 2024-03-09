#![allow(dead_code)]
use super::{
    netlist2::{Capacitor, LayerInfo, Net, NetInfo, NetType, Netlist, Node, NodeType, Resistor},
    nomutil::{empty_or_comment, float, identifier, optionally_quoted_string, ws},
};
use color_eyre;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{char, digit1, line_ending, not_line_ending, one_of},
    combinator::{map, opt, value, verify},
    multi::{many0, many1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    FindSubstring, IResult, Parser,
};
use std::{collections::HashMap, fs};

#[derive(Debug)]
pub struct Dspf {
    pub info: DspfInfo,
    pub filesize: u64,
    pub netlist: Netlist,
}

#[derive(Debug)]
pub struct DspfInfo {
    version: String,
    header: HashMap<String, String>,
    subckt: Subckt,
    ground_nets: Vec<String>,
    layer_map: Option<HashMap<u8, String>>,
}

#[derive(Debug)]
struct Subckt {
    name: String,
    ports: Vec<String>,
}

impl Dspf {
    pub fn load(file_path: &str) -> color_eyre::Result<Dspf> {
        let filesize = fs::metadata(file_path)?.len();
        let data = fs::read_to_string(file_path)?;
        let (mut tail, info) = parse_dspf_info(&data).map_err(|err| err.to_owned())?;

        let mut instance_sections: Vec<(usize, &str)> = Vec::new();

        // temporary map to look up node index when parsing R/C instances
        let mut nodes_map: HashMap<String, usize> = HashMap::new();

        let mut netlist = Netlist::default();
        for ground_name in &info.ground_nets {
            let node_idx = netlist.add_node(Node {
                name: ground_name.to_owned(),
                info: NodeType::Ground,
                coord: None,
                capacitors: Vec::new(),
                of_net: 0, // will override below
            });
            nodes_map.insert(ground_name.to_owned(), node_idx);
            let net_idx = netlist.add_net(Net {
                info: NetInfo {
                    name: ground_name.to_owned(),
                    net_type: NetType::GroundNode,
                },
                total_capacitance: f64::NAN,
                sub_nets: vec![node_idx],
                resistors: Vec::new(),
            });
            netlist.all_nodes[node_idx].of_net = net_idx;
        }

        loop {
            let (t, (net, nodes)) =
                read_net_block(tail, &info.subckt.ports).map_err(|err| err.to_owned())?;

            let net_name = net.info.name.clone();

            let net_idx = netlist.add_net(net);
            if nodes.len() == 0 {
                // special case, if the net has no subnodes listed explicitly,
                // we need to insert the net itself as its only subnode
                let node_idx = netlist.add_node(Node {
                    name: net_name.clone(),
                    info: NodeType::Other,
                    coord: None,
                    capacitors: Vec::new(),
                    of_net: 0, // will override below
                });
                nodes_map.insert(net_name.clone(), node_idx);
                netlist.all_nodes[node_idx].of_net = net_idx;
            }

            for node in nodes {
                let name = node.name.clone();
                let node_idx = netlist.add_node(node);
                nodes_map.insert(name, node_idx);
            }

            // capture everything after this net section (until the next *|NET or end of subckt),
            // store it away and skip ahead
            if let Some(n) = t.find_substring("\n*|NET") {
                instance_sections.push((123, &t[..n]));
                tail = &t[n + 1..];
            } else if let Some(n) = t.find_substring("\n.ENDS") {
                instance_sections.push((123, &t[..n]));
                // tail = &t[n + 1..];
                break;
            } else {
                panic!("No .ENDS statement found");
            }
        }

        let layer_map_inv: HashMap<String, u8> = HashMap::from_iter(
            info.layer_map.as_ref().unwrap().iter().map(|(k, v)| (v.clone(), *k)),
        );

        for (_x, inst_slice) in instance_sections {
            let (_, instances) = parse_instances(inst_slice).map_err(|err| err.to_owned())?;
            for inst in instances {
                match inst {
                    ElementDef::R {
                        nodes,
                        value,
                        layer,
                    } => {
                        let r = Resistor {
                            nodes: (nodes_map[&nodes.0], nodes_map[&nodes.1]),
                            value,
                            layer: layer_map_inv[&layer.unwrap()],
                        };
                        let net = netlist.all_nodes[r.nodes.0].of_net;
                        netlist.all_nets[net].resistors.push(r);
                    }
                    ElementDef::C {
                        nodes,
                        value,
                        layers,
                    } => {
                        let c = Capacitor {
                            nodes: (nodes_map[&nodes.0], nodes_map[&nodes.1]),
                            value,
                            layers,
                        };
                        let nodes = c.nodes.clone();
                        netlist.capacitors.push(c);
                        let cap_idx = netlist.capacitors.len() - 1;
                        netlist.all_nodes[nodes.0].capacitors.push(cap_idx);
                        netlist.all_nodes[nodes.1].capacitors.push(cap_idx);
                    }
                }
            }
        }

        Ok(Dspf {
            info,
            filesize,
            netlist,
        })
    }
}

#[test]
fn test_dspf() -> color_eyre::Result<()> {
    let file_path = "DSPF/nmos_trcp70.dspf";

    let _dspf = Dspf::load(file_path)?;
    // dbg!(dspf);

    Ok(())
}

fn parse_dspf_info(input: &str) -> IResult<&str, DspfInfo> {
    let (tail, ((version, header), subckt, (ground_nets, layer_map))) =
        tuple((parse_header, parse_subckt, parse_ground_and_layers)).parse(input)?;

    let dspf_info = DspfInfo {
        version,
        header,
        subckt,
        ground_nets,
        layer_map,
    };
    Ok((tail, dspf_info))
}

#[test]
fn test_dspf_info() -> color_eyre::Result<()> {
    let file_path = "DSPF/nmos_trcp70_trunc.dspf";
    // let file_path = "DSPF/dcdc_ps_250mohm_trcp70.dspf";
    // let file_path = "DSPF/dcdc_error_amp_trcp70.dspf";
    let data = fs::read_to_string(file_path)?;

    let (tail, _info) = parse_dspf_info(&data).map_err(|err| err.to_owned())?;

    dbg!(&tail[..50]);

    Ok(())
}

fn parse_spf_version(input: &str) -> IResult<&str, String> {
    let version = ws(alt((tag("1.0"), tag("1.3"), tag("1.5"))));

    let (tail, s) = delimited(tag("*|DSPF"), version, line_ending)(input)?;
    Ok((tail, s.to_string()))
}

fn parse_info_strings(input: &str) -> IResult<&str, HashMap<String, String>> {
    let tags = (
        tag("DESIGN"),
        tag("DATE"),
        tag("VENDOR"),
        tag("PROGRAM"),
        tag("VERSION"),
        tag("DIVIDER"),
        tag("DELIMITER"),
        tag("DeviceFingerDelim"),
        tag("BUSBIT"),
        tag("GLOBAL_TEMPERATURE"),
        tag("OPERATING_TEMPERATURE"),
    );

    let (tail, lines) = many0(preceded(
        empty_or_comment,
        delimited(
            tag("*|"),
            tuple((ws(alt(tags)), optionally_quoted_string)),
            line_ending,
        ),
    ))
    .parse(input)?;

    let info: HashMap<String, String> =
        lines.iter().map(|l| (l.0.to_string(), l.1.to_string())).collect();

    Ok((tail, info))
}

fn parse_header(input: &str) -> IResult<&str, (String, HashMap<String, String>)> {
    tuple((
        delimited(empty_or_comment, parse_spf_version, empty_or_comment),
        terminated(parse_info_strings, empty_or_comment),
    ))
    .parse(input)
}

fn parse_subckt(input: &str) -> IResult<&str, Subckt> {
    let (tail, (name, ports)) = delimited(
        tag(".SUBCKT"),
        tuple((ws(identifier), many0(ws(identifier)))),
        line_ending,
    )
    .parse(input)?;
    Ok((
        tail,
        Subckt {
            name: name.to_string(),
            ports: ports.iter().map(|s| s.to_string()).collect(),
        },
    ))
}

fn parse_ground_net(input: &str) -> IResult<&str, String> {
    let (tail, net_name) = delimited(tag("*|GROUND_NET"), ws(identifier), line_ending)(input)?;
    Ok((tail, net_name.to_string()))
}

fn parse_layer_map(input: &str) -> IResult<&str, HashMap<u8, String>> {
    let (tail, layer_pairs) = preceded(
        pair(tag("*LAYER_MAP"), line_ending),
        many1(delimited(
            tag("*"),
            pair(digit1, ws(identifier)),
            line_ending,
        )),
    )(input)?;

    Ok((
        tail,
        layer_pairs
            .iter()
            .map(|(i, name)| (i.parse::<u8>().unwrap(), name.to_string()))
            .collect(),
    ))
}

fn parse_ground_and_layers(
    input: &str,
) -> IResult<&str, (Vec<String>, Option<HashMap<u8, String>>)> {
    let (tail, grounds) = preceded(empty_or_comment, many1(parse_ground_net))(input)?;
    let (tail, layer_map) =
        delimited(empty_or_comment, opt(parse_layer_map), empty_or_comment)(tail)?;

    Ok((tail, (grounds, layer_map)))
}

// --------------------------------------------------------
//  NET BLOCKS
// --------------------------------------------------------

#[derive(Debug)]
struct NetDef {
    name: String,
    cap: f64,
}

fn parse_net_def(input: &str) -> IResult<&str, NetDef> {
    let (tail, (name, cap)) =
        delimited(tag("*|NET"), pair(ws(identifier), ws(float)), line_ending)(input)?;
    Ok((
        tail,
        NetDef {
            name: name.to_string(),
            cap,
        },
    ))
}

fn slash_comment(input: &str) -> IResult<&str, String> {
    preceded(tag("//"), not_line_ending)
        .parse(input)
        .map(|(tail, s)| (tail, s.to_string()))
}

#[derive(Clone)]
enum NodeLetter {
    P,
    I,
    S,
}

// parse *|P, *|I or *|S statement
fn parse_nodedef(input: &str) -> IResult<&str, Node> {
    let (mut tail, (which, name)) = preceded(
        tag("*|"),
        separated_pair(
            alt((
                value(NodeLetter::P, char('P')),
                value(NodeLetter::I, char('I')),
                value(NodeLetter::S, char('S')),
            )),
            ws(char('(')),
            ws(identifier),
        ),
    )
    .parse(input)?;

    // parser for the end of the line (shared between the 3 types)
    let mut ending = terminated(
        separated_pair(
            opt(pair(ws(float), ws(float))),
            ws(char(')')),
            opt(slash_comment),
        ),
        line_ending,
    );

    let coord: Option<(f64, f64)>;
    let _comment: Option<String>;

    let info = match which {
        NodeLetter::P => {
            let (t, (pin_type, pin_cap)) = pair(ws(one_of("IOBXSJ")), ws(float)).parse(tail)?;
            (tail, (coord, _comment)) = ending.parse(t)?;
            NodeType::SubcktPin { pin_type, pin_cap }
        }
        NodeLetter::I => {
            let (t, (inst_name, pin_name, pin_type, pin_cap)) = tuple((
                ws(identifier),
                ws(identifier),
                ws(one_of("IOBXSJ")),
                ws(float),
            ))
            .parse(tail)?;

            (tail, (coord, _comment)) = ending.parse(t)?;
            NodeType::InstPin {
                inst_name: inst_name.to_string(),
                pin_name: pin_name.to_string(),
                pin_type,
                pin_cap,
            }
        }
        NodeLetter::S => {
            (tail, (coord, _comment)) = ending.parse(tail)?;
            NodeType::Other
        }
    };

    let node = Node {
        name: name.to_string(),
        info,
        coord,
        capacitors: Vec::new(),
        of_net: 0,
    };

    Ok((tail, node))
}

fn parse_nodedefs(input: &str) -> IResult<&str, Vec<Node>> {
    many0(parse_nodedef)(input)
}

fn read_net_block<'a>(
    input: &'a str,
    subckt_pins: &Vec<String>,
) -> IResult<&'a str, (Net, Vec<Node>)> {
    let (tail, (net_def, nodedefs)) = pair(parse_net_def, parse_nodedefs)(input)?;

    // TODO: we are assuming that ground nodes can't have a net block...
    // otherwise we would have to check here if the net is a ground.
    let typ: NetType;
    if subckt_pins.contains(&net_def.name) {
        typ = NetType::SubcktPin;
    } else {
        typ = NetType::Other;
    }

    Ok((
        tail,
        (
            Net {
                info: NetInfo {
                    name: net_def.name,
                    net_type: typ,
                },
                total_capacitance: net_def.cap,
                sub_nets: Vec::new(),
                resistors: Vec::new(),
            },
            nodedefs,
        ),
    ))
}

enum ElementDef {
    R {
        nodes: (String, String),
        value: f64,
        layer: Option<String>,
    },
    C {
        nodes: (String, String),
        value: f64,
        layers: LayerInfo,
    },
}

fn parse_dollar_params(input: &str) -> IResult<&str, Vec<(String, String)>> {
    many1(map(
        ws(preceded(
            char('$'),
            separated_pair(identifier, char('='), is_not(" \t\n")),
        )),
        |(a, b)| (a.to_string(), b.to_string()),
    ))(input)
}

fn parse_resistor(input: &str) -> IResult<&str, ElementDef> {
    let (tail, (_name, nodes, value, layer, _params)) = tuple((
        verify(ws(identifier), |s: &str| s.starts_with("R")),
        map(pair(ws(identifier), ws(identifier)), |(a, b)| {
            (a.to_string(), b.to_string())
        }),
        ws(float),
        opt(ws(preceded(char('$'), identifier))),
        parse_dollar_params,
    ))(input)?;

    Ok((
        tail,
        ElementDef::R {
            nodes,
            value,
            layer: layer.map(|s| s.to_string()),
        },
    ))
}

fn parse_capacitor(input: &str) -> IResult<&str, ElementDef> {
    let (tail, (_name, nodes, value, params)) = tuple((
        verify(ws(identifier), |s: &str| s.starts_with("C")),
        map(pair(ws(identifier), ws(identifier)), |(a, b)| {
            (a.to_string(), b.to_string())
        }),
        ws(float),
        parse_dollar_params,
    ))(input)?;

    let layers: LayerInfo;
    if let Some((_, layer)) = params.iter().find(|(name, _)| name == "lvl") {
        layers = LayerInfo::Single(layer.parse().unwrap());
    } else if let Some((_, layer1)) = params.iter().find(|(name, _)| name == "lvl1") {
        if let Some((_, layer2)) = params.iter().find(|(name, _)| name == "lvl2") {
            layers = LayerInfo::Pair(layer1.parse().unwrap(), layer2.parse().unwrap());
        } else {
            panic!("matching $lvl2 not found in capacitor")
        }
    } else {
        layers = LayerInfo::None;
    }

    Ok((
        tail,
        ElementDef::C {
            nodes,
            value,
            layers,
        },
    ))
}

fn parse_instances(input: &str) -> IResult<&str, Vec<ElementDef>> {
    many0(terminated(
        alt((parse_resistor, parse_capacitor)),
        line_ending,
    ))(input)
}
