use ratatui::{
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
};

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

// https://docs.rs/ratatui/latest/src/ratatui/widgets/gauge.rs.html#221
fn get_unicode_block<'a>(frac: f64) -> &'a str {
    match (frac * 8.0).round() as u16 {
        1 => symbols::block::ONE_EIGHTH,
        2 => symbols::block::ONE_QUARTER,
        3 => symbols::block::THREE_EIGHTHS,
        4 => symbols::block::HALF,
        5 => symbols::block::FIVE_EIGHTHS,
        6 => symbols::block::THREE_QUARTERS,
        7 => symbols::block::SEVEN_EIGHTHS,
        8 => symbols::block::FULL,
        _ => " ",
    }
}

pub fn line_bar(width: usize, frac: f64) -> Line<'static> {
    if width < 2 || !frac.is_finite() {
        return Line::from(" ");
    }

    let width = width - 2;

    // reversed direction: use 1-frac and inver the color...
    let frac = 1.0 - frac.clamp(0.0, 1.0);

    let bar_width = frac * width as f64;
    let mut bar = symbols::block::FULL.repeat(bar_width.floor() as usize);
    bar.push_str(get_unicode_block(bar_width % 1.0));
    let space = " ".repeat(width - bar_width.floor() as usize - 1);
    let color = Color::Rgb(
        ((1.0 - frac).sqrt().sqrt() * 255.0) as u8,
        (frac * 255.0) as u8,
        0,
    );
    let color = Style::new().fg(color).add_modifier(Modifier::REVERSED);

    Line::from(vec![
        Span::raw("│"),
        Span::raw(bar).style(color),
        Span::raw(space).style(color),
        Span::raw("│"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn eng_format(value: f64) -> String {
        eng_format_scale(value, value)
    }

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
