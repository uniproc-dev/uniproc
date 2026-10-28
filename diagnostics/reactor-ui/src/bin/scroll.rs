use std::time::Duration;
use windows_reactor::*;

struct Row;

impl Component for Row {
    type Input = (usize, u32);
    type Message = ();

    fn create(_: &Self::Input, _: &ComponentContext<Self>) -> Self {
        Self
    }

    fn view(&self, input: &Self::Input, _: &mut ViewContext<Self>) -> View {
        row(input.0, input.1)
    }
}

fn row(index: usize, tick: u32) -> View {
    Border::new()
        .automation_id(format!("row-{index}"))
        .height(32.0)
        .background(if index.is_multiple_of(2) {
            Color::rgb(235, 235, 235)
        } else {
            Color::rgb(255, 255, 255)
        })
        .content(
            TextBlock::new()
                .text(format!("Row {index:04}  tick {tick}"))
                .foreground(Color::rgb(0, 0, 0)),
        )
        .into()
}

struct Probe {
    #[cfg(native_probe)]
    _focus: windows_core::EventRevoker,
    focus: ElementRef<Button>,
    tick: u32,
    mode: String,
    timer: Option<ComponentTimer>,
}

impl Component for Probe {
    type Input = ();
    type Message = ();

    fn create(_: &(), cx: &ComponentContext<Self>) -> Self {
        Self {
            #[cfg(native_probe)]
            _focus: reactor_ui_repros::probe::install(),
            focus: ElementRef::new(),
            tick: 0,
            mode: std::env::var("SCROLL_MODE").unwrap_or_else(|_| "component".into()),
            timer: Some(cx.set_timeout(Duration::from_secs(1), ())),
        }
    }

    fn update(&mut self, _: (), cx: &ComponentContext<Self>) {
        self.tick += 1;
        #[cfg(native_probe)]
        reactor_ui_repros::probe::scroll(self.tick);
        self.timer = Some(cx.set_timeout(Duration::from_secs(1), ()));
    }

    fn view(&self, _: &(), cx: &mut ViewContext<Self>) -> View {
        let focus = self.focus.clone();
        cx.use_effect("focus", (), move || {
            let _ = focus.request_focus();
            None
        });
        let tick = self.tick;
        let mode = self.mode.clone();
        let content: View = if mode == "stack" {
            StackPanel::new()
                .children((0..1024).map(|index| row(index, tick)).collect::<Vec<_>>())
                .into()
        } else {
            ItemsRepeater::new()
                .virtual_source(VirtualSource::new(
                    0,
                    std::env::var("SCROLL_COUNT")
                        .ok()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(1024),
                    |index| index,
                    move |index| {
                        eprintln!("REALIZE {index} tick={tick}");
                        if std::env::var_os("SCROLL_PADDING").is_some() && index >= 1001 {
                            Border::new().into()
                        } else if mode == "component" {
                            View::component::<Row>((index, tick))
                        } else {
                            row(index, tick)
                        }
                    },
                ))
                .into()
        };
        let content = Grid::new()
            .width(640.0)
            .height(480.0)
            .rows([GridLength::Auto, GridLength::Star(1.0)])
            .children((
                Button::new()
                    .element_ref(&self.focus)
                    .content(TextBlock::new().text(format!(
                        "{}: fixed 32px rows, tick {}",
                        self.mode, self.tick
                    ))),
                ScrollViewer::new()
                    .grid_row(1)
                    .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
                    .content(content),
            ));
        cx.window_frame(format!("Scroll reproduction: {}", self.mode), content)
    }
}

fn main() {
    App::run_component::<Probe>(()).unwrap();
}
