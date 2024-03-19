use crate::{app::Action, event::Event};
use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::NetInfo;
use dspf_parse::dspf::netlist::ResReport;
use dspf_parse::dspf::Dspf;
use ratatui::prelude::*;
use ratatui::Frame;
use std::rc::Rc;

use super::multi_node_selection::MultiNodeSelectionWidget;
use super::net_selection::NetSelectionWidget;
use super::res_result::ResResultWidget;
use super::status_bar::StatusBar;
use super::Render;

#[derive(PartialEq)]
enum FocusUI {
    Inputs,
    Outputs,
    Result,
}

pub struct ResMainUI {
    dspf: Rc<Dspf>,
    selected_net: Option<String>,
    net_selection_widget: NetSelectionWidget,
    input_selection_widget: MultiNodeSelectionWidget,
    output_selection_widget: MultiNodeSelectionWidget,
    result_widget: ResResultWidget,
    focus: FocusUI,
}

impl ResMainUI {
    pub fn new(dspf: Rc<Dspf>) -> Self {
        let mut nets: Vec<NetInfo> =
            dspf.netlist.all_nets.iter().map(|net| net.info.clone()).collect();
        nets.sort_by_key(|info| (info.net_type.clone(), info.name.clone()));

        Self {
            dspf,
            selected_net: None,
            net_selection_widget: NetSelectionWidget::new(nets, "Select net:", true),
            input_selection_widget: MultiNodeSelectionWidget::default(),
            output_selection_widget: MultiNodeSelectionWidget::default(),
            result_widget: ResResultWidget::default(),
            focus: FocusUI::Inputs,
        }
    }

    fn tab(&mut self) {
        use FocusUI::*;
        self.focus = match self.focus {
            Inputs => Outputs,
            Outputs => Result,
            Result => Inputs,
        };
        self.highlight_focused()
    }

    fn left(&mut self) {
        use FocusUI::*;
        self.focus = match self.focus {
            Inputs => Inputs,
            Outputs => Inputs,
            Result => Outputs,
        };
        self.highlight_focused()
    }

    fn right(&mut self) {
        use FocusUI::*;
        self.focus = match self.focus {
            Inputs => Outputs,
            Outputs => Result,
            Result => Result,
        };
        self.highlight_focused()
    }

    fn highlight_focused(&mut self) {
        self.input_selection_widget.focus = self.focus == FocusUI::Inputs;
        self.output_selection_widget.focus = self.focus == FocusUI::Outputs;
        self.result_widget.focus = self.focus == FocusUI::Result;
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::SelectNet(net) => {
                self.selected_net = net;

                if let Some(net_name) = &self.selected_net {
                    let idx = self.dspf.netlist.nets_map[net_name];
                    let net = &self.dspf.netlist.all_nets[idx];
                    let nodes: Vec<_> =
                        net.subnodes.iter().map(|idx| &self.dspf.netlist.all_nodes[*idx]).collect();
                    self.input_selection_widget =
                        MultiNodeSelectionWidget::new(nodes.clone(), "Input node(s):");
                    self.output_selection_widget =
                        MultiNodeSelectionWidget::new(nodes.clone(), "Output node(s):");

                    let excluded_nodes: Vec<_> = self.input_selection_widget.menu.items.clone();
                    self.output_selection_widget.exclude(excluded_nodes);
                }
            }
            Action::NodesChanged => {
                if self.selected_net.is_some() {
                    let excluded_nodes: Vec<_> = self.input_selection_widget.menu.items.clone();
                    self.output_selection_widget.exclude(excluded_nodes);

                    if !self.input_selection_widget.menu.items.is_empty()
                        && !self.output_selection_widget.menu.items.is_empty()
                    {
                        self.analyze();
                    }
                }
            }
            _ => {}
        }
    }

    fn analyze(&mut self) {
        if let Some(net) = &self.selected_net {
            let inputs: Vec<_> = self
                .input_selection_widget
                .menu
                .items
                .iter()
                .map(|info| info.name.clone())
                .collect();
            let outputs: Vec<_> = self
                .output_selection_widget
                .menu
                .items
                .iter()
                .map(|info| info.name.clone())
                .collect();

            let report = self
                .dspf
                .netlist
                .get_path_resistance(net, inputs.as_slice(), outputs.as_slice())
                .unwrap_or(ResReport::default());

            self.result_widget = ResResultWidget::new(report)
        }
    }
}

impl Render for ResMainUI {
    fn render(&mut self, frame: &mut Frame) {
        let mut status_bar = StatusBar::default()
            .top_left("dspf-analyzer")
            .bottom_left(&self.dspf.as_ref().file_path);
        frame.render_widget(&mut status_bar, frame.size());

        match &self.selected_net {
            None => {
                frame.render_widget(&mut self.net_selection_widget, status_bar.inner);
            }
            Some(_) => {
                let cols_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![
                        Constraint::Fill(1),
                        Constraint::Fill(1),
                        Constraint::Fill(2),
                    ])
                    .split(status_bar.inner);

                frame.render_widget(&mut self.input_selection_widget, cols_layout[0]);
                frame.render_widget(&mut self.output_selection_widget, cols_layout[1]);
                frame.render_widget(&mut self.result_widget, cols_layout[2])
            }
        }
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
                        KeyCode::Enter => {
                            match self.selected_net {
                                None => {
                                    let action = self.net_selection_widget.handle_event(event);
                                    self.handle_action(action);
                                }
                                Some(_) => {}
                            }
                            Action::None
                        }

                        // delegate others to the currently focused widget
                        _ => {
                            let action = match self.selected_net {
                                None => self.net_selection_widget.handle_event(event),
                                Some(_) => match self.focus {
                                    FocusUI::Inputs => {
                                        self.input_selection_widget.handle_event(event)
                                    }
                                    FocusUI::Outputs => {
                                        self.output_selection_widget.handle_event(event)
                                    }
                                    FocusUI::Result => self.result_widget.handle_event(event),
                                },
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
