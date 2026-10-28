#[cfg(native_probe)]
mod measurement {
    use std::time::Duration;
    use windows_reactor::*;

    const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="1" y="1" width="22" height="22" rx="3" fill="none" stroke="#101010" stroke-width="1"/><circle cx="12" cy="12" r="7" fill="none" stroke="#101010" stroke-width=".7"/><path d="M3 21L21 3M4 6h16M4 9h16M4 15h16M4 18h16" fill="none" stroke="#101010" stroke-width=".35"/></svg>"##;

    struct Probe {
        _focus: windows_core::EventRevoker,
        focus: ElementRef<Button>,
        phase: u32,
        timer: Option<ComponentTimer>,
        shared: EncodedImage,
        fresh: Vec<EncodedImage>,
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
                _focus: reactor_ui_repros::probe::install(),
                focus: ElementRef::new(),
                phase: 0,
                timer: Some(cx.set_timeout(Duration::from_secs(4), ())),
                shared: EncodedImage::new(SVG.to_vec()),
                fresh: (0..4).map(|_| EncodedImage::new(SVG.to_vec())).collect(),
            }
        }

        fn update(&mut self, _: (), cx: &ComponentContext<Self>) {
            self.phase += 1;
            if self.phase == 1 {
                reactor_ui_repros::probe::capture();
            } else if reactor_ui_repros::probe::poll_captures() {
                self.timer = None;
                let _ = cx.close_window();
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
            let content: View = {
                let mut cells = Vec::<View>::new();
                for (i, size) in sizes.iter().enumerate() {
                    cells.push(
                        StackPanel::new()
                            .grid_row(0)
                            .grid_column(i as i32)
                            .children((
                                TextBlock::new()
                                    .text(format!("Shared {size}"))
                                    .foreground(Color::rgb(16, 16, 16)),
                                image(&self.shared, *size),
                            ))
                            .into(),
                    );
                    cells.push(
                        StackPanel::new()
                            .grid_row(1)
                            .grid_column(i as i32)
                            .children((
                                TextBlock::new()
                                    .text(format!("Independent {size}"))
                                    .foreground(Color::rgb(16, 16, 16)),
                                image(&self.fresh[i], *size),
                            ))
                            .into(),
                    );
                }
                Grid::new()
                    .width(960.0)
                    .height(600.0)
                    .columns([GridLength::Star(1.0); 4])
                    .rows([GridLength::Star(1.0); 2])
                    .background(Color::rgb(255, 255, 255))
                    .children(cells)
                    .into()
            };
            cx.window_frame(
                "Immediate SVG measurement",
                StackPanel::new().children((
                    Button::new()
                        .element_ref(&self.focus)
                        .content("Measurement"),
                    content,
                )),
            )
        }
    }

    pub fn run() {
        App::run_component::<Probe>(()).unwrap();
    }
}

fn main() {
    #[cfg(native_probe)]
    measurement::run();
    #[cfg(not(native_probe))]
    eprintln!("Build with WINUI_WINMD to enable the initial SVG measurement");
}
