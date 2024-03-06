// enum NetCap {
//     ForAggressor { name: String, cap: f64 },
//     ForLayer {},
// }

trait TableCell: PartialOrd {
    fn fmt(&self) -> String;
    // fn sort_key(&self) -> &impl Ord;
}

// impl<T> TableCell for T {
//     fn fmt(&self) -> String {
//         format!("{}", self.0)
//     }
// }

#[derive(PartialEq, PartialOrd)]
struct NumberCell(f64);
impl TableCell for NumberCell {
    fn fmt(&self) -> String {
        format!("{:.3e}", self.0)
    }
}

#[derive(PartialEq, PartialOrd)]
struct TextCell(String);
impl TableCell for TextCell {
    fn fmt(&self) -> String {
        format!("{}", self.0)
    }
}

struct TableRow {
    name: TextCell,
    value: NumberCell,
}

struct Table(Vec<TableRow>);

impl Table {
    fn sort(&mut self, column: usize) {
        match column {
            0 => self.0.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap()),
            1 => self
                .0
                .sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap()),
            _ => {}
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let table = Table(vec![
            TableRow {
                name: TextCell(String::from("bar")),
                value: NumberCell(10.2),
            },
            TableRow {
                name: TextCell(String::from("foo")),
                value: NumberCell(1230.2),
            },
        ]);
    }
}

// pub fn net_cap_report(netlist: &Netlist, net: &str) -> Result<NetCapReport> {
//     let net = netlist.get_net(net)?;
//     for subnode in net.sub_nets.iter() {
//         // for p in subnode.
//     }

//     Ok(NetCapReport { nets: vec![] })
// }
