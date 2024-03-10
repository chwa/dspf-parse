pub fn eng_format(value: f64) -> String {
    eng_format_scale(value, value)
}

// version that prints a number in the same format as the (larger) `value_for_scale`
pub fn eng_format_scale(value: f64, value_for_scale: f64) -> String {
    let map: [(i32, char); 10] = [
        (-18, 'a'),
        (-15, 'f'),
        (-12, 'p'),
        (-9, 'n'),
        (-6, 'u'),
        (-3, 'm'),
        (0, ' '),
        (3, 'k'),
        (6, 'M'),
        (9, 'G'),
    ];
    let mut log = value_for_scale.abs().log10();
    if log.is_infinite() {
        log = 0.0;
    }

    let option = map.into_iter().find(|(exp, _)| (*exp as f64) > log - 3.0).unwrap_or((0, ' '));
    let mant = value / 10.0_f64.powf(option.0 as f64);
    let log_int = log.floor() as i32;
    let suffix = option.1;

    format!(
        "{mant:>5.prec$} {suffix}F",
        prec = (3 + option.0 - log_int) as usize
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eng_format() {
        assert_eq!(&eng_format(0.0), "0.000  F");
        assert_eq!(&eng_format(1.0001), "1.000  F");
        assert_eq!(&eng_format(0.9999), "999.9 mF");
        assert_eq!(&eng_format(-0.9999), "-999.9 mF");
        assert_eq!(&eng_format(123.98), "124.0  F");
        assert_eq!(&eng_format(-123.98), "-124.0  F");
        assert_eq!(&eng_format(888.06e-15), "888.1 fF");
        assert_eq!(&eng_format(-888.06e-15), "-888.1 fF");
        assert_eq!(&eng_format(0.2388e9), "238.8 MF");
        assert_eq!(&eng_format(-0.2388e9), "-238.8 MF");
    }
}
