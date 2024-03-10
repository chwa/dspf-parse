use crate::util::{eng_format, eng_format_scale};
use crate::{app::Action, event::Event};
use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::NetCapReport;
use ratatui::{prelude::*, widgets::*};

use super::main_menu::TableSelect;
use super::net_cap_main::focus_style;

pub struct NetCapResultWidget {
    pub focus: bool,
    report: NetCapReport,
    menu: TableSelect<String>,
    menu_height: u16,
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

impl NetCapResultWidget {
    pub fn new(report: NetCapReport) -> Self {
        let mut ui = Self {
            focus: false,
            report,
            menu: TableSelect::new(vec![]),
            menu_height: 1,
        };

        ui.update_list();
        ui
    }

    fn update_list(&mut self) {
        let mut names: Vec<String> =
            self.report.table.iter().map(|i| i.aggressor_name.clone()).collect();

        names.insert(0, String::from(""));
        self.menu = TableSelect::new(names);
        self.menu.select_state(Some(0));
    }
}
impl Widget for &mut NetCapResultWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(2), Constraint::Fill(1)])
            .split(area);
        self.menu_height = rows_layout[1].as_size().height - 2;
        let fs = focus_style(self.focus);

        Paragraph::new("\n  Aggressor net:").style(fs.1).render(rows_layout[0], buf);

        let mut rows: Vec<_> = self
            .report
            .table
            .iter()
            .map(|x| {
                let col1 = Line::raw(&x.aggressor_name);
                let col2 = Line::raw(eng_format_scale(x.cap, self.report.total_cap));
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

        let widths = [
            Constraint::Fill(2),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(7),
        ];

        StatefulWidget::render(
            Table::new(rows, widths)
                .block(
                    Block::new()
                        .borders(Borders::ALL)
                        .border_type(fs.0)
                        .padding(Padding::horizontal(1)),
                )
                .highlight_style(Style::new().reversed()),
            rows_layout[1],
            buf,
            &mut self.menu.state,
        );
    }
}

impl NetCapResultWidget {
    fn handle_arrow(&mut self, code: KeyCode) -> Action {
        let pos = match code {
            KeyCode::Up => self.menu.up(1),
            KeyCode::Down => self.menu.down(1),
            KeyCode::PageUp => self.menu.up((self.menu_height - 1).into()),
            KeyCode::PageDown => self.menu.down((self.menu_height - 1).into()),
            _ => 0, // not possible
        };
        let aggressor_net_name = self.menu.items[pos].to_owned();
        match aggressor_net_name.len() {
            0 => Action::SelectNet(self.report.net_name.clone()),
            _ => Action::SelectNetPair(self.report.net_name.clone(), aggressor_net_name),
        }
    }
    pub fn handle_event(&mut self, event: &Event) -> Action {
        match event {
            Event::Tick => Action::None,
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown => {
                            self.handle_arrow(key_event.code)
                        }
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
