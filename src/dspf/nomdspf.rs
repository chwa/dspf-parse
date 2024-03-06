use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    ops::Sub,
};

#[allow(unused_imports)]
use color_eyre::Result;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, alphanumeric1, line_ending, not_line_ending, space0},
    combinator::{not, recognize},
    error::ParseError,
    multi::{many0, many0_count},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult, InputTake, Parser,
};

fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(space0, inner, space0)
}

fn parse_spf_version(input: &str) -> IResult<&str, &str> {
    let version = ws(alt((tag("1.0"), tag("1.3"), tag("1.5"))));

    delimited(tuple((tag("*|"), ws(tag("DSPF")))), version, line_ending)(input)
}

fn qstring(input: &str) -> IResult<&str, &str> {
    ws(delimited(tag("\""), is_not("\"\n"), tag("\""))).parse(input)
}
fn optionally_quoted_string(input: &str) -> IResult<&str, &str> {
    ws(alt((
        delimited(tag("\""), is_not("\"\n"), tag("\"")),
        is_not("\"\n"),
    )))
    .parse(input)
}

fn comment_lines(input: &str) -> IResult<&str, Vec<&str>> {
    many0(delimited(
        tag("*").and(not(tag("|"))),
        not_line_ending,
        line_ending,
    ))
    .parse(input)
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

    let (remaining, lines) = many0(preceded(
        comment_lines,
        delimited(
            tag("*|"),
            tuple((ws(alt(tags)), optionally_quoted_string)),
            line_ending,
        ),
    ))
    .parse(input)?;

    let info: HashMap<String, String> = lines
        .iter()
        .map(|l| (l.0.to_string(), l.1.to_string()))
        .collect();

    Ok((remaining, info))
}

fn parse_header(input: &str) -> IResult<&str, (&str, HashMap<String, String>)> {
    tuple((
        delimited(comment_lines, parse_spf_version, comment_lines),
        terminated(parse_info_strings, comment_lines),
    ))
    .parse(input)
}

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(alpha1, many0_count(alt((alphanumeric1, tag("_")))))).parse(input)
}

fn parse_subckt(input: &str) -> IResult<&str, (&str, Vec<&str>)> {
    delimited(
        tag(".SUBCKT"),
        tuple((ws(identifier), many0(ws(identifier)))),
        line_ending,
    )
    .parse(input)
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
}

impl DspfInfo {
    pub fn from_str(input: &str) -> Result<Self> {
        let header = parse_header(input).unwrap(); // TODO: how to use ? here without capturing 'input'
                                                   // dbg!(header);

        let (subckt_name, subckt_ports) = parse_subckt(header.0.take(50)).unwrap().1;
        Ok(DspfInfo {
            version: header.1 .0.to_string(),
            header: header.1 .1,
            subckt: Subckt {
                name: subckt_name.to_string(),
                ports: subckt_ports.iter().map(|s| s.to_string()).collect(),
            },
        })
    }
}

#[test]
fn test_from_file() -> Result<()> {
    let file_path = "DSPF/nmos_trcp70.dspf";
    let mut f = File::open(file_path).unwrap();
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    let (rest, header) = parse_header(&buffer).unwrap();

    assert_eq!(header.0, "1.5");
    assert_eq!(header.1["PROGRAM"], "Cadence Quantus Extraction");
    assert_eq!(rest.take(7), ".SUBCKT");

    Ok(())
}

#[test]
fn test_dspfinfo() -> Result<()> {
    let file_path = "DSPF/nmos_trcp70.dspf";
    let mut f = File::open(file_path).unwrap();
    let mut buffer = String::new();

    f.read_to_string(&mut buffer)?;

    let info = DspfInfo::from_str(&buffer);
    dbg!(info);

    Ok(())
}
