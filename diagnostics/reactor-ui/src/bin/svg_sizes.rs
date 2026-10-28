use std::time::Duration;
use windows_reactor::*;

const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="1" y="1" width="22" height="22" rx="3" fill="none" stroke="#101010" stroke-width="1"/><circle cx="12" cy="12" r="7" fill="none" stroke="#101010" stroke-width=".7"/><path d="M3 21L21 3M4 6h16M4 9h16M4 15h16M4 18h16" fill="none" stroke="#101010" stroke-width=".35"/></svg>"##;

struct Probe {
    #[cfg(native_probe)]
    _focus: windows_core::EventRevoker,
    focus: ElementRef<Button>,
    phase: u32,
    shared: EncodedImage,
    fresh: Vec<EncodedImage>,
    ready: bool,
    timer: Option<ComponentTimer>,
}

fn image(source: &EncodedImage, size: f64) -> View {
    Image::new()
        .source_svg(source.clone())
        .width(size)
        .height(size)
        .into()
}

impl Component for Probe {
    type Input = ();
    type Message = ();

    fn create(_: &(), cx: &ComponentContext<Self>) -> Self {
        Self {
            #[cfg(native_probe)]
            _focus: reactor_ui_repros::probe::install(),
            focus: ElementRef::new(),
            phase: 0,
            shared: EncodedImage::new(SVG.to_vec()),
            fresh: (0..4).map(|_| EncodedImage::new(SVG.to_vec())).collect(),
            ready: false,
            timer: Some(cx.set_timeout(Duration::from_secs(5), ())),
        }
    }

    fn update(&mut self, _: (), cx: &ComponentContext<Self>) {
        self.phase += 1;
        self.ready = true;
        if self.phase == 1 {
            self.timer = cfg!(native_probe).then(|| cx.set_timeout(Duration::from_secs(3), ()));
            return;
        }
        #[cfg(native_probe)]
        if self.phase == 2 {
            reactor_ui_repros::probe::capture();
        }
        #[cfg(native_probe)]
        if self.phase > 2 && reactor_ui_repros::probe::poll_captures() {
            self.timer = None;
            return;
        }
        self.timer = Some(cx.set_timeout(Duration::from_millis(200), ()));
    }

    fn view(&self, _: &(), cx: &mut ViewContext<Self>) -> View {
        let focus = self.focus.clone();
        cx.use_effect("focus", (), move || {
            let _ = focus.request_focus();
            None
        });
        let sizes = [24.0, 48.0, 96.0, 192.0];
        let content: View = if self.ready {
            let mut cells = vec![
                TextBlock::new()
                    .text("Shared source prewarmed at first size")
                    .into(),
            ];
            for (i, size) in sizes.iter().enumerate() {
                cells.push(
                    StackPanel::new()
                        .grid_row(1)
                        .grid_column(i as i32)
                        .children((
                            TextBlock::new().text(format!("Shared {size}")),
                            image(&self.shared, *size),
                        ))
                        .into(),
                );
                cells.push(
                    StackPanel::new()
                        .grid_row(2)
                        .grid_column(i as i32)
                        .children((
                            TextBlock::new().text(format!("Independent {size}")),
                            image(&self.fresh[i], *size),
                        ))
                        .into(),
                );
            }
            Grid::new()
                .width(960.0)
                .height(600.0)
                .columns([GridLength::Star(1.0); 4])
                .rows([
                    GridLength::Auto,
                    GridLength::Star(1.0),
                    GridLength::Star(1.0),
                ])
                .background(Color::rgb(255, 255, 255))
                .children(cells)
                .into()
        } else {
            let size = if std::env::var_os("SVG_LARGE_FIRST").is_some() {
                192.0
            } else {
                24.0
            };
            image(&self.shared, size)
        };
        cx.window_frame(
            "SVG size reproduction",
            StackPanel::new().children((
                Button::new().element_ref(&self.focus).content("SVG probe"),
                content,
            )),
        )
    }
}

fn main() {
    App::run_component::<Probe>(()).unwrap();
}
