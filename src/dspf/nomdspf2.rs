use super::{
    netlist::Netlist,
    nomutil::{empty_or_comment, float, identifier, optionally_quoted_string, ws},
};
use color_eyre::Result;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1, line_ending, not_line_ending, one_of},
    combinator::{map_res, opt},
    multi::{many0, many1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, Parser,
};
use std::{collections::HashMap, fs};

#[derive(Default, Debug)]
pub struct Dspf {
    pub file_path: String,
    pub file_size: u64,
    pub netlist: Option<Netlist>,
}

impl Dspf {
    pub fn load(file_path: &str) -> Result<Dspf> {
        let size = fs::metadata(file_path)?.len();
        let data = fs::read_to_string(file_path)?;
        Ok(Dspf {
            file_path: String::from("path"),
            file_size: size,
            netlist: Some(Netlist::new()),
        })
    }
}

#[test]
fn test_dspf() -> Result<()> {
    let file_path = "DSPF/nmos_trcp70.dspf";

    let dspf = Dspf::load(file_path)?;
    dbg!(dspf);
    // let (rest, header) = parse_header(&buffer).unwrap();

    // assert_eq!(header.0, "1.5");
    // assert_eq!(header.1["PROGRAM"], "Cadence Quantus Extraction");
    // assert_eq!(rest.take(7), ".SUBCKT");

    Ok(())
}

#[derive(Debug)]
struct Subckt {
    name: String,
    ports: Vec<String>,
}

#[derive(Debug)]
pub struct DspfInfo {
    version: String,
    header: HashMap<String, String>,
    subckt: Subckt,
    ground_nets: Vec<String>,
    layer_map: Option<HashMap<i32, String>>,
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
fn test_dspf_info() -> Result<()> {
    let file_path = "DSPF/nmos_trcp70_trunc.dspf";
    // let file_path = "DSPF/dcdc_ps_250mohm_trcp70.dspf";
    let data = fs::read_to_string(file_path)?;

    let (tail, info) = parse_dspf_info(&data).map_err(|err| err.to_owned())?;

    dbg!(info);

    let (tail, net) = parse_net_block(tail).map_err(|err| err.to_owned())?;
    dbg!(net);
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

fn parse_layer_map(input: &str) -> IResult<&str, HashMap<i32, String>> {
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
            .map(|(i, name)| (i.parse::<i32>().unwrap(), name.to_string()))
            .collect(),
    ))
}

fn parse_ground_and_layers(
    input: &str,
) -> IResult<&str, (Vec<String>, Option<HashMap<i32, String>>)> {
    let (tail, grounds) = preceded(empty_or_comment, many1(parse_ground_net))(input)?;
    let (tail, layer_map) =
        delimited(empty_or_comment, opt(parse_layer_map), empty_or_comment)(tail)?;

    Ok((tail, (grounds, layer_map)))
}

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

#[derive(Debug)]
struct PinDef {
    name: String,
    pin_type: char,
    pin_cap: f64,
    coord: Option<(f64, f64)>,
    comment: Option<String>,
}

fn slash_comment(input: &str) -> IResult<&str, String> {
    preceded(tag("//"), not_line_ending)
        .parse(input)
        .map(|(tail, s)| (tail, s.to_string()))
}

fn parse_pin_def(input: &str) -> IResult<&str, PinDef> {
    let (tail, ((name, pin_type, pin_cap, coord), comment)) = delimited(
        tag("*|P"),
        pair(
            delimited(
                ws(char('(')),
                tuple((
                    ws(identifier),
                    ws(one_of("IOBXSJ")),
                    ws(float),
                    opt(pair(ws(float), ws(float))),
                )),
                ws(char(')')),
            ),
            opt(slash_comment),
        ),
        line_ending,
    )
    .parse(input)?;

    Ok((
        tail,
        PinDef {
            name: name.to_string(),
            pin_type,
            pin_cap,
            coord,
            comment,
        },
    ))
}
fn parse_pin_defs(input: &str) -> IResult<&str, Vec<PinDef>> {
    many0(parse_pin_def)(input)
}

#[derive(Debug)]
struct InstPinDef {
    inst_pin_name: String,
    inst_name: String,
    pin_name: String,
    pin_type: char,
    pin_cap: f64,
    coord: Option<(f64, f64)>,
    comment: Option<String>,
}

fn parse_instpin_def(input: &str) -> IResult<&str, InstPinDef> {
    let (tail, ((inst_pin_name, inst_name, pin_name, pin_type, pin_cap, coord), comment)) =
        delimited(
            tag("*|I"),
            pair(
                delimited(
                    ws(char('(')),
                    tuple((
                        ws(identifier),
                        ws(identifier),
                        ws(identifier),
                        ws(one_of("IOBXSJ")),
                        ws(float),
                        opt(pair(ws(float), ws(float))),
                    )),
                    ws(char(')')),
                ),
                opt(slash_comment),
            ),
            line_ending,
        )
        .parse(input)?;

    Ok((
        tail,
        InstPinDef {
            inst_pin_name: inst_pin_name.to_string(),
            inst_name: inst_name.to_string(),
            pin_name: pin_name.to_string(),
            pin_type,
            pin_cap,
            coord,
            comment,
        },
    ))
}
fn parse_instpin_defs(input: &str) -> IResult<&str, Vec<InstPinDef>> {
    many0(parse_instpin_def)(input)
}

#[derive(Debug)]
struct SubnodeDef {
    name: String,
    coord: Option<(f64, f64)>,
    comment: Option<String>,
}

fn parse_subnode_def(input: &str) -> IResult<&str, SubnodeDef> {
    let (tail, ((name, coord), comment)) = delimited(
        tag("*|S"),
        pair(
            delimited(
                ws(char('(')),
                tuple((ws(identifier), opt(pair(ws(float), ws(float))))),
                ws(char(')')),
            ),
            opt(slash_comment),
        ),
        line_ending,
    )
    .parse(input)?;

    Ok((
        tail,
        SubnodeDef {
            name: name.to_string(),
            coord,
            comment,
        },
    ))
}
fn parse_subnode_defs(input: &str) -> IResult<&str, Vec<SubnodeDef>> {
    many0(parse_subnode_def)(input)
}
// fn parse_instpin_defs(input: &str) -> IResult<&str, String> {}

// fn parse_subnode_defs(input: &str) -> IResult<&str, String> {}

fn parse_net_block(
    input: &str,
) -> IResult<&str, (NetDef, Vec<PinDef>, Vec<InstPinDef>, Vec<SubnodeDef>)> {
    tuple((
        parse_net_def,
        parse_pin_defs,
        parse_instpin_defs,
        parse_subnode_defs,
    ))(input)
}
