use crate::{app::Action, event::Event};
use crossterm::event::KeyCode;
use dspf_parse::dspf::netlist::NetInfo;
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
}

pub struct ResMainUI {
    dspf: Rc<Dspf>,
    selected_net: Option<String>,
    net_selection_widget: NetSelectionWidget,
    node_selection_widgets: Option<(MultiNodeSelectionWidget, MultiNodeSelectionWidget)>,
    result_widget: ResResultWidget,
    focus: FocusUI,
}

impl ResMainUI {
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

        Self {
            dspf,
            selected_net: None,
            net_selection_widget: NetSelectionWidget::new(nets, "Select net:", true),
            node_selection_widgets: None,
            result_widget: ResResultWidget::default(),
            focus: FocusUI::Inputs,
        }
    }

    fn tab(&mut self) {
        use FocusUI::*;
        self.focus = match self.focus {
            Inputs => Outputs,
            Outputs => Inputs,
        };
        self.highlight_focused()
    }

    fn left(&mut self) {
        self.focus = FocusUI::Inputs;
        self.highlight_focused()
    }

    fn right(&mut self) {
        self.focus = FocusUI::Outputs;
        self.highlight_focused()
    }

    fn highlight_focused(&mut self) {
        if let Some(widgets) = self.node_selection_widgets.as_mut() {
            widgets.0.focus = self.focus == FocusUI::Inputs;
            widgets.1.focus = self.focus == FocusUI::Outputs;
        }
    }

    fn handle_action(&mut self, action: Action) {
        if let Action::SelectNet(net) = action {
            self.selected_net = net;

            if let Some(net_name) = &self.selected_net {
                let netlist = self.dspf.netlist.as_ref().unwrap();
                let idx = netlist.nets_map[net_name];
                let net = &netlist.all_nets[idx];
                let nodes: Vec<_> =
                    net.subnodes.iter().map(|idx| &netlist.all_nodes[*idx]).collect();
                self.node_selection_widgets = Some((
                    MultiNodeSelectionWidget::new(nodes.clone(), "Input node(s):"),
                    MultiNodeSelectionWidget::new(nodes, "Output node(s):"),
                ));
            }
        }
    }

    fn analyze(&mut self) {
        if let Some(widgets) = self.node_selection_widgets.as_ref() {
            let inputs: Vec<_> =
                widgets.0.menu.items.iter().map(|info| info.name.clone()).collect();
            let outputs: Vec<_> =
                widgets.1.menu.items.iter().map(|info| info.name.clone()).collect();

            let netlist = self.dspf.netlist.as_ref().unwrap();

            let report = netlist
                .get_path_resistance(
                    self.selected_net.as_ref().unwrap(),
                    &inputs[0],
                    outputs.as_slice(),
                )
                .unwrap();

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

                if let Some(widgets) = self.node_selection_widgets.as_mut() {
                    frame.render_widget(&mut widgets.0, cols_layout[0]);
                    frame.render_widget(&mut widgets.1, cols_layout[1]);
                }

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
                        KeyCode::Esc => Action::Esc,
                        KeyCode::Enter => {
                            match self.selected_net {
                                None => {
                                    let action = self.net_selection_widget.handle_event(event);
                                    self.handle_action(action);
                                }
                                Some(_) => {
                                    self.analyze();
                                }
                            }
                            Action::None
                        }

                        // delegate others to the currently focused widget
                        _ => {
                            let action: Action;
                            if self.selected_net.is_none() {
                                action = self.net_selection_widget.handle_event(event);
                            } else if let Some(widgets) = self.node_selection_widgets.as_mut() {
                                action = match self.focus {
                                    FocusUI::Inputs => widgets.0.handle_event(event),
                                    FocusUI::Outputs => widgets.1.handle_event(event),
                                };
                            } else {
                                action = Action::None;
                            }

                            // self.handle_action(action);

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
