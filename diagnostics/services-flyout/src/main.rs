use std::time::Duration;
use windows_reactor::*;

#[derive(Clone, Copy, PartialEq)]
enum Command {
    Start,
    Stop,
    Restart,
}

impl Command {
    fn label(self) -> &'static str {
        match self {
            Self::Start => "Starting",
            Self::Stop => "Stopping",
            Self::Restart => "Restarting",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Scenario {
    Normal,
    Slow,
    Denied,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Stopping,
    Starting,
    Waiting,
    Done,
    Failed,
}

struct Operation {
    row: usize,
    command: Command,
    scenario: Scenario,
    phase: Phase,
    checkpoint: u32,
    waiting_from: Phase,
}

impl Operation {
    fn busy(&self) -> bool {
        !matches!(self.phase, Phase::Done | Phase::Failed)
    }

    fn stage(&self) -> &'static str {
        match self.phase {
            Phase::Stopping => "Stopping service…",
            Phase::Starting => "Starting service…",
            Phase::Waiting => "The service is taking longer to respond…",
            Phase::Done => "Operation completed",
            Phase::Failed => "Access is denied.",
        }
    }
}

struct Scratchpad {
    names: Vec<String>,
    running: Vec<bool>,
    selected: usize,
    scenario: Scenario,
    operation: Option<Operation>,
    expanded: bool,
    details: bool,
    timer: Option<ComponentTimer>,
}

enum Message {
    Select(usize),
    Scenario(Scenario),
    Begin(Command),
    Step,
    Toggle,
    Details,
    Dismiss,
}

fn text(value: impl AsRef<str>) -> TextBlock {
    TextBlock::new().text(value).font_size(14.0)
}

impl Component for Scratchpad {
    type Input = ();
    type Message = Message;

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            names: (0..120)
                .map(|index| {
                    if index == 0 {
                        "Uniproc UI test service".into()
                    } else {
                        format!("Example service {index:03}")
                    }
                })
                .collect(),
            running: (0..120).map(|index| index % 3 != 1).collect(),
            selected: 0,
            scenario: Scenario::Normal,
            operation: None,
            expanded: false,
            details: false,
            timer: None,
        }
    }

    fn update(&mut self, message: Message, cx: &ComponentContext<Self>) {
        match message {
            Message::Select(row) => self.selected = row,
            Message::Scenario(scenario) => self.scenario = scenario,
            Message::Toggle => self.expanded = !self.expanded,
            Message::Details => self.details = !self.details,
            Message::Dismiss => {
                if self.operation.as_ref().is_some_and(|op| !op.busy()) {
                    self.operation = None;
                    self.expanded = false;
                }
            }
            Message::Begin(command) => {
                if self.operation.as_ref().is_some_and(Operation::busy) {
                    return;
                }
                if (command == Command::Start) == self.running[self.selected] {
                    return;
                }
                self.operation = Some(Operation {
                    row: self.selected,
                    command,
                    scenario: self.scenario,
                    phase: if command == Command::Start {
                        Phase::Starting
                    } else {
                        Phase::Stopping
                    },
                    checkpoint: 1,
                    waiting_from: Phase::Stopping,
                });
                self.expanded = true;
                self.timer = Some(cx.set_timeout(Duration::from_secs(1), Message::Step));
            }
            Message::Step => {
                self.timer = None;
                let Some(op) = self.operation.as_mut().filter(|op| op.busy()) else {
                    return;
                };
                if op.scenario == Scenario::Denied {
                    op.phase = Phase::Failed;
                } else if op.phase == Phase::Waiting {
                    op.phase = op.waiting_from;
                    op.checkpoint += 1;
                    self.timer = Some(cx.set_timeout(Duration::from_secs(1), Message::Step));
                } else if op.scenario == Scenario::Slow
                    && op.checkpoint == 2
                    && op.phase != Phase::Waiting
                {
                    op.waiting_from = op.phase;
                    op.phase = Phase::Waiting;
                    self.timer = Some(cx.set_timeout(Duration::from_secs(8), Message::Step));
                } else if op.checkpoint < 3 {
                    op.checkpoint += 1;
                    self.timer = Some(cx.set_timeout(Duration::from_secs(1), Message::Step));
                } else if op.command == Command::Restart && op.phase != Phase::Starting {
                    self.running[op.row] = false;
                    op.phase = Phase::Starting;
                    op.checkpoint = 1;
                    self.timer = Some(cx.set_timeout(Duration::from_secs(1), Message::Step));
                } else {
                    self.running[op.row] = op.command != Command::Stop;
                    op.phase = Phase::Done;
                }
            }
        }
    }

    fn view(&self, _: &(), cx: &mut ViewContext<Self>) -> View {
        let busy = self.operation.as_ref().is_some_and(Operation::busy);
        let commands: [View; 3] =
            [Command::Start, Command::Stop, Command::Restart].map(|command| {
                Button::new()
                    .content(match command {
                        Command::Start => "Start",
                        Command::Stop => "Stop",
                        Command::Restart => "Restart",
                    })
                    .is_enabled(
                        !busy && ((command == Command::Start) != self.running[self.selected]),
                    )
                    .on_click(cx.callback(move |_| Message::Begin(command)))
                    .into()
            });
        let scenarios: [View; 3] = [
            (Scenario::Normal, "Normal"),
            (Scenario::Slow, "Slow response"),
            (Scenario::Denied, "Access denied"),
        ]
        .map(|(scenario, label)| {
            Button::new()
                .content(text(label).foreground(if self.scenario == scenario {
                    ThemeBrush::AccentText
                } else {
                    ThemeBrush::PrimaryText
                }))
                .on_click(cx.callback(move |_| Message::Scenario(scenario)))
                .into()
        });
        let rows = self
            .names
            .iter()
            .enumerate()
            .map(|(row, name)| {
                let status = match self
                    .operation
                    .as_ref()
                    .filter(|op| op.row == row && op.busy() && op.scenario != Scenario::Denied)
                {
                    Some(op)
                        if op.phase == Phase::Starting
                            || op.phase == Phase::Waiting && op.waiting_from == Phase::Starting =>
                    {
                        "Starting"
                    }
                    Some(_) => "Stopping",
                    None if self.running[row] => "Running",
                    None => "Stopped",
                };
                Button::new()
                    .background(if row == self.selected {
                        ThemeBrush::CardBackground
                    } else {
                        ThemeBrush::SolidBackground
                    })
                    .horizontal_alignment(HorizontalAlignment::Stretch)
                    .horizontal_content_alignment(HorizontalAlignment::Stretch)
                    .content(
                        Grid::new()
                            .columns([GridLength::Star(1.0), GridLength::Pixel(130.0)])
                            .children((text(name).grid_column(0), text(status).grid_column(1))),
                    )
                    .on_click(cx.callback(move |_| Message::Select(row)))
                    .into()
            })
            .collect::<Vec<View>>();
        let base = Grid::new()
            .rows([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
            .children((
                StackPanel::new().spacing(12.0).grid_row(0).children((
                    text("Services · flyout scratchpad").font_size(20.0),
                    text("Simulated services. No Windows services are changed.").opacity(0.65),
                    StackPanel::new()
                        .orientation(Orientation::Horizontal)
                        .spacing(8.0)
                        .children(scenarios),
                    StackPanel::new()
                        .orientation(Orientation::Horizontal)
                        .spacing(8.0)
                        .children(commands),
                )),
                ScrollViewer::new()
                    .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
                    .vertical_scroll_bar_visibility(ScrollBarVisibility::Auto)
                    .margin(Thickness::xy(0.0, 20.0))
                    .grid_row(1)
                    .content(StackPanel::new().spacing(4.0).children(rows)),
                Button::new()
                    .grid_row(2)
                    .horizontal_alignment(HorizontalAlignment::Right)
                    .is_enabled(self.operation.is_some())
                    .content(
                        self.operation
                            .as_ref()
                            .map(|op| format!("{} · {}", self.names[op.row], op.stage()))
                            .unwrap_or_else(|| "No operations".into()),
                    )
                    .on_click(cx.callback(|_| Message::Toggle)),
            ));
        let mut layers = vec![base.into()];
        if let Some(op) = self.operation.as_ref().filter(|_| self.expanded) {
            let mut content: Vec<View> = vec![
                text(format!("{} {}", op.command.label(), self.names[op.row]))
                    .text_wrapping(TextWrapping::Wrap)
                    .into(),
                text(op.stage())
                    .text_wrapping(TextWrapping::Wrap)
                    .foreground(if op.phase == Phase::Failed {
                        ThemeBrush::SystemCritical
                    } else {
                        ThemeBrush::PrimaryText
                    })
                    .into(),
            ];
            if op.busy() {
                content.push(ProgressBar::new().is_indeterminate(true).into());
            }
            if self.details {
                content.push(
                    text(format!(
                        "Simulated SCM checkpoint: {}\nWait hint: 5 seconds",
                        op.checkpoint
                    ))
                    .opacity(0.65)
                    .into(),
                );
            }
            content.push(
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(8.0)
                    .children((
                        Button::new()
                            .content(if self.details {
                                "Hide details"
                            } else {
                                "Details"
                            })
                            .on_click(cx.callback(|_| Message::Details)),
                        Button::new()
                            .content("Collapse")
                            .on_click(cx.callback(|_| Message::Toggle)),
                        Button::new()
                            .content("Dismiss")
                            .is_enabled(!op.busy())
                            .on_click(cx.callback(|_| Message::Dismiss)),
                    ))
                    .into(),
            );
            layers.push(
                Border::new()
                    .padding(20.0)
                    .corner_radius(CornerRadius::new(8.0, 8.0, 0.0, 0.0))
                    .background(ThemeBrush::SolidBackground)
                    .border_brush(ThemeBrush::CardStroke)
                    .border_thickness(1.0)
                    .horizontal_alignment(HorizontalAlignment::Stretch)
                    .vertical_alignment(VerticalAlignment::Bottom)
                    .content(StackPanel::new().spacing(14.0).children(content))
                    .into(),
            );
        }
        cx.window_frame(
            "Services flyout scratchpad",
            Grid::new()
                .min_width(640.0)
                .min_height(580.0)
                .margin(20.0)
                .children(layers),
        )
    }
}

fn main() {
    App::run_component::<Scratchpad>(()).unwrap();
}
