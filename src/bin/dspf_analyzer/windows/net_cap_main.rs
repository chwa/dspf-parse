use crate::{app::Action, event::Event};
use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::{AggrNet, LayerCapReport, NetCapReport, NetInfo};
use dspf_parse::dspf::Dspf;
use ratatui::Frame;
use ratatui::{prelude::*, widgets::*};
use std::rc::Rc;

use super::layer_cap_result::LayerCapResultWidget;
use super::net_cap_result::NetCapResultWidget;
use super::net_selection::NetSelectionWidget;
use super::status_bar::StatusBar;
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
        let mut nets: Vec<NetInfo> =
            dspf.netlist.all_nets.iter().map(|net| net.info.clone()).collect();
        nets.sort_by_key(|info| (info.net_type.clone(), info.name.clone()));

        let mut ui = Self {
            dspf,
            net_selection_widget: NetSelectionWidget::new(nets, "Victim net:", false),
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
            Result => Layers,
            Layers => Selection,
        };
        self.highlight_focused()
    }

    fn left(&mut self) {
        use FocusUI::*;
        self.focus = match self.focus {
            Selection => Selection,
            Result => Selection,
            Layers => Result,
        };
        self.highlight_focused()
    }

    fn right(&mut self) {
        use FocusUI::*;
        self.focus = match self.focus {
            Selection => Result,
            Result => Layers,
            Layers => Layers,
        };
        self.highlight_focused()
    }

    fn highlight_focused(&mut self) {
        self.net_selection_widget.focus = self.focus == FocusUI::Selection;
        self.net_cap_result_widget.focus = self.focus == FocusUI::Result;
        self.layer_cap_result_widget.focus = self.focus == FocusUI::Layers;
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::SelectNet(net) => {
                let report = match &net {
                    Some(net_name) => self
                        .dspf
                        .netlist
                        .get_net_capacitors(net_name)
                        .unwrap_or(NetCapReport::default()),
                    None => NetCapReport::default(),
                };
                let layer_report = match net {
                    Some(net_name) => self
                        .dspf
                        .netlist
                        .get_layer_capacitors(&net_name, AggrNet::Total)
                        .unwrap_or(LayerCapReport::default()),
                    None => LayerCapReport::default(),
                };
                self.net_cap_result_widget = NetCapResultWidget::new(report);
                self.layer_cap_result_widget = LayerCapResultWidget::new(layer_report);
                self.highlight_focused();
            }
            Action::SelectAggrNet(aggr_net) => {
                if let Some(net_name) = self.net_selection_widget.selected() {
                    let report_layers = match aggr_net {
                        Some(aggr) => self
                            .dspf
                            .netlist
                            .get_layer_capacitors(&net_name, aggr)
                            .unwrap_or(LayerCapReport::default()),
                        None => LayerCapReport::default(),
                    };
                    self.layer_cap_result_widget = LayerCapResultWidget::new(report_layers);
                    self.highlight_focused();
                }
            }

            _ => {}
        }
    }
}

impl Render for NetCapMainUI {
    fn render(&mut self, frame: &mut Frame) {
        let mut status_bar = StatusBar::default()
            .top_left("dspf-analyzer")
            .bottom_left(&self.dspf.as_ref().file_path);
        frame.render_widget(&mut status_bar, frame.size());

        let cols_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Fill(2),
                Constraint::Fill(2),
            ])
            .split(status_bar.inner);

        frame.render_widget(&mut self.net_selection_widget, cols_layout[0]);

        // self.selection_ui.render_in_rect(frame, &cols_layout[0]);
        frame.render_widget(&mut self.net_cap_result_widget, cols_layout[1]);
        frame.render_widget(&mut self.layer_cap_result_widget, cols_layout[2]);
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
                        KeyCode::Esc => Action::MainMenu,

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
