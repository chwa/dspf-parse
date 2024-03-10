use crate::util::eng_format;
use crate::{app::Action, event::Event};
use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::NetCapReport;
use ratatui::Frame;
use ratatui::{prelude::*, widgets::*};

use super::Render;

pub struct NetCapResultUI {
    report: NetCapReport,
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
    if width < 2 || !frac.is_finite() || frac < 0.0_f64 || frac > 1.0_f64 {
        return Line::from(" ");
    }

    // reversed direction: use 1-frac and inver the color...
    let frac = 1.0 - frac;

    let bar_width = frac * width as f64;
    let mut bar = symbols::block::FULL.repeat(bar_width.floor() as usize);
    bar.push_str(get_unicode_block(bar_width % 1.0));
    let space = " ".repeat(width - bar_width.floor() as usize - 1);
    let color = Color::Rgb(
        ((1.0 - frac).sqrt().sqrt() * 255.0) as u8,
        (frac * 255.0) as u8,
        0,
    );

    Line::from(vec![Span::raw(bar), Span::raw(space)]).style(Style::new().fg(color).reversed())
}

impl NetCapResultUI {
    pub fn new(report: NetCapReport) -> Self {
        let ui = Self { report };
        ui
    }

    pub fn render_in_rect(&mut self, frame: &mut Frame, rect: &Rect) -> () {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(2), Constraint::Fill(1)])
            .split(*rect);

        frame.render_widget(Paragraph::new("\n  Aggressor net:"), rows_layout[0]);

        let mut rows: Vec<_> = self
            .report
            .table
            .iter()
            .map(|x| {
                let col1 = Line::raw(&x.aggressor_name);
                let col2 = Line::raw(eng_format(x.cap));
                let col3 = line_bar(8, x.cap / self.report.total_cap);
                let col4 = Line::raw(format!("{:6.1}%", 100.0 * x.cap / self.report.total_cap));
                Row::new(vec![col1, col2, col3, col4])
            })
            .collect();

        let col1 = Line::raw("[TOTAL]");
        let col2 = Line::raw(eng_format(self.report.total_cap));
        let col3 = Line::raw("");
        let col4 = Line::raw(format!("{:6.1}%", 100.0));
        rows.insert(
            0,
            Row::new(vec![col1, col2, col3, col4]).style(Style::new().add_modifier(Modifier::BOLD)),
        );
        // let rows = [
        //     Row::new(vec!["abc1", "def1", "ghi1"]),
        //     Row::new(vec!["abc2", "def2", "ghi2"]),
        //     Row::new(vec!["abc3", "def3", "ghi3"]),
        // ];
        let widths = [
            Constraint::Fill(2),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(7),
        ];
        let table = Table::new(rows, widths).block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .padding(Padding::horizontal(1)),
        );
        frame.render_widget(table, rows_layout[1]);
    }
}

impl Render for NetCapResultUI {
    fn render(&mut self, frame: &mut Frame) -> () {
        self.render_in_rect(frame, &frame.size());
    }

    fn handle_event(&mut self, event: &Event) -> Action {
        match event {
            Event::Tick => Action::None,
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => Action::Quit,

                        _ => Action::None,
                    }
                } else {
                    Action::None
                }
            }
            Event::Mouse(_) => Action::None,
            Event::Resize(_, _) => Action::None,
        }
    }
}
