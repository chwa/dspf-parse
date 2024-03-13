use crate::{app::Action, event::Event};
use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::{AggrNet, LayerCapReport, NetCapReport, NetInfo};
use dspf_parse::dspf::Dspf;
use ratatui::Frame;
use ratatui::{prelude::*, widgets::*};
use std::rc::Rc;

use super::layer_cap_result::LayerCapResultWidget;
use super::net_cap_result::NetCapResultWidget;
use super::net_cap_selection::NetSelectionWidget;
use super::Render;

#[derive(PartialEq)]
enum FocusUI {
    Selection,
    Result,
    Layers,
}

pub fn focus_style(focus: bool) -> (BorderType, Style) {
    match focus {
        true => (BorderType::Thick, Style::new().bold()),
        false => (BorderType::Rounded, Style::new()),
    }
}

pub struct NetCapMainUI {
    dspf: Rc<Dspf>,
    net_selection_widget: NetSelectionWidget,
    net_cap_result_widget: NetCapResultWidget,
    layer_cap_result_widget: LayerCapResultWidget,
    focus: FocusUI,
}

impl NetCapMainUI {
    pub fn new(dspf: Rc<Dspf>) -> Self {
        let mut nets: Vec<NetInfo> = dspf
            .netlist
            .as_ref()
            .unwrap()
            .all_nets
            .iter()
            .map(|net| net.info.clone())
            .collect();
        nets.sort_by_key(|info| (info.net_type.clone(), info.name.clone()));

        let mut ui = Self {
            dspf,
            net_selection_widget: NetSelectionWidget::new(nets),
            net_cap_result_widget: NetCapResultWidget::new(NetCapReport::default()),
            layer_cap_result_widget: LayerCapResultWidget::new(LayerCapReport::default()),
            focus: FocusUI::Selection,
        };
        ui.highlight_focused();

        // trigger the update of the net_cap_result_widget
        let action = ui.net_selection_widget.update_list();
        ui.handle_action(action);
        ui
    }

    fn tab(&mut self) {
        use FocusUI::*;
        self.focus = match self.focus {
            Selection => Result,
            Result => Selection, // skip layers for now
            Layers => Selection,
        };
        self.highlight_focused()
    }

    fn left(&mut self) {
        self.focus = FocusUI::Selection;
        self.highlight_focused()
    }

    fn right(&mut self) {
        self.focus = FocusUI::Result;
        self.highlight_focused()
    }

    fn highlight_focused(&mut self) {
        self.net_selection_widget.focus = self.focus == FocusUI::Selection;
        self.net_cap_result_widget.focus = self.focus == FocusUI::Result;
        self.layer_cap_result_widget.focus = self.focus == FocusUI::Layers;
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::SelectVictimNet(net) => {
                let report = match &net {
                    Some(net_name) => {
                        self.dspf.netlist.as_ref().unwrap().get_net_capacitors(net_name).unwrap()
                    }
                    None => NetCapReport::default(),
                };
                let layer_report = match net {
                    Some(net_name) => self
                        .dspf
                        .netlist
                        .as_ref()
                        .unwrap()
                        .get_layer_capacitors(&net_name, AggrNet::Total)
                        .unwrap(),
                    None => LayerCapReport::default(),
                };
                self.net_cap_result_widget = NetCapResultWidget::new(report);
                self.layer_cap_result_widget = LayerCapResultWidget::new(layer_report);
                self.highlight_focused();
            }
            Action::SelectAggrNet(aggr_net) => {
                let report_layers = match aggr_net {
                    Some(aggr) => self
                        .dspf
                        .netlist
                        .as_ref()
                        .unwrap()
                        .get_layer_capacitors(&self.net_selection_widget.selected().unwrap(), aggr)
                        .unwrap(),
                    None => LayerCapReport::default(),
                };
                self.layer_cap_result_widget = LayerCapResultWidget::new(report_layers);
                self.highlight_focused();
            }

            _ => {}
        }
    }
}

impl Render for NetCapMainUI {
    fn render(&mut self, frame: &mut Frame) {
        let rows_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .split(frame.size());
        let cols_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Fill(2),
                Constraint::Fill(2),
            ])
            .split(rows_layout[1]);

        // let titles = vec![" Victim net: ", " Aggressor net: ", " Layer pairs: "];

        // let selected = match self.focus {
        //     FocusUI::Selection => 0,
        //     FocusUI::Result => 1,
        //     FocusUI::Layers => 2,
        // };

        // let block_inner: Vec<_> = titles
        //     .iter()
        //     .enumerate()
        //     .map(|(i, title)| {
        //         let b = Block::default()
        //             .title(*title)
        //             .title_alignment(Alignment::Center)
        //             .title_style(match i == selected {
        //                 true => Style::new().bold(),
        //                 false => Style::new(),
        //             })
        //             .borders(Borders::ALL)
        //             .border_type(match i == selected {
        //                 true => BorderType::Thick,
        //                 false => BorderType::Plain,
        //             });
        //         let inner = b.inner(cols_layout[i]);
        //         frame.render_widget(b, cols_layout[i]);
        //         inner
        //     })
        //     .collect();

        frame.render_widget(&mut self.net_selection_widget, cols_layout[0]);

        // self.selection_ui.render_in_rect(frame, &cols_layout[0]);
        frame.render_widget(&mut self.net_cap_result_widget, cols_layout[1]);
        frame.render_widget(&mut self.layer_cap_result_widget, cols_layout[2]);

        let header = vec![Line::from("dspf_analyzer v0.0.0")];

        frame.render_widget(
            Paragraph::new(header).style(Style::new().reversed()).alignment(Alignment::Left),
            rows_layout[0],
        );
        let footer = vec![Line::from(self.dspf.as_ref().file_path.clone())];
        // let text = vec![Line::from("This is just the status bar.")];

        frame.render_widget(
            Paragraph::new(footer).style(Style::new().reversed()).alignment(Alignment::Left),
            rows_layout[2],
        );
    }

    fn handle_event(&mut self, event: &Event) -> Action {
        match event {
            Event::Tick => Action::None,
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Left => {
                            self.left();
                            Action::None
                        }
                        KeyCode::Right => {
                            self.right();
                            Action::None
                        }
                        KeyCode::Tab => {
                            self.tab();
                            Action::None
                        }
                        KeyCode::Esc => Action::Esc,

                        // delegate others to the currently focused widget
                        _ => {
                            let action = match self.focus {
                                FocusUI::Selection => self.net_selection_widget.handle_event(event),
                                FocusUI::Result => self.net_cap_result_widget.handle_event(event),
                                FocusUI::Layers => self.layer_cap_result_widget.handle_event(event),
                            };
                            self.handle_action(action);
                            Action::None
                        }
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
