#[cfg(native_probe)]
mod experiment {
    use reactor_ui_repros::{bindings as c, probe};
    use std::time::Duration;
    use windows_core::Interface;
    use windows_reactor::*;

    const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="1" y="1" width="22" height="22" rx="3" fill="none" stroke="#101010" stroke-width="1"/><circle cx="12" cy="12" r="7" fill="none" stroke="#101010" stroke-width=".7"/><path d="M3 21L21 3M4 6h16M4 9h16M4 15h16M4 18h16" fill="none" stroke="#101010" stroke-width=".35"/></svg>"##;

    fn automatic() -> windows_core::Result<c::SvgImageSource> {
        let source: c::SvgImageSource =
            windows_core::factory::<c::SvgImageSource, windows_core::imp::IGenericFactory>()?
                .ActivateInstance()?;
        let stream: c::InMemoryRandomAccessStream = windows_core::factory::<
            c::InMemoryRandomAccessStream,
            windows_core::imp::IGenericFactory,
        >()?
        .ActivateInstance()?;
        let writer = c::DataWriter::CreateDataWriter(&stream.GetOutputStreamAt(0)?)?;
        writer.WriteBytes(SVG)?;
        let image = source.clone();
        writer.StoreAsync()?.when(move |result| {
            result.unwrap();
            writer.DetachStream().unwrap();
            stream.Seek(0).unwrap();
            image
                .SetSourceAsync(&stream)
                .unwrap()
                .when(|result| eprintln!("AUTO_LOAD {result:?}"))
                .unwrap();
        })?;
        Ok(source)
    }

    struct Experiment {
        _focus: windows_core::EventRevoker,
        focus: ElementRef<Button>,
        sources: Vec<c::SvgImageSource>,
        refs: Vec<ElementRef<Image>>,
        reactor: EncodedImage,
        phase: u32,
        timer: Option<ComponentTimer>,
    }

    impl Component for Experiment {
        type Input = ();
        type Message = ();

        fn create(_: &(), cx: &ComponentContext<Self>) -> Self {
            Self {
                _focus: probe::install(),
                focus: ElementRef::new(),
                sources: Vec::new(),
                refs: (0..9).map(|_| ElementRef::new()).collect(),
                reactor: EncodedImage::new(SVG.to_vec()),
                phase: 0,
                timer: Some(cx.set_timeout(Duration::from_secs(1), ())),
            }
        }

        fn update(&mut self, _: (), cx: &ComponentContext<Self>) {
            self.phase += 1;
            if self.phase == 1 {
                self.sources = vec![automatic().unwrap()];
                for size in [24.0, 96.0, 192.0, 384.0, 768.0, 1536.0, 3072.0] {
                    self.sources
                        .push(reactor_ui_repros::bounded_svg(SVG, size, size).unwrap());
                }
                self.sources
                    .push(reactor_ui_repros::bounded_svg(SVG, 3072.0, 3072.0).unwrap());
                self.timer = Some(cx.set_timeout(Duration::from_secs(4), ()));
            } else {
                if self.phase == 2 {
                    probe::capture();
                } else if probe::poll_captures() {
                    self.timer = None;
                    let _ = cx.close_window();
                    return;
                }
                self.timer = Some(cx.set_timeout(Duration::from_millis(200), ()));
            }
        }

        fn view(&self, _: &(), cx: &mut ViewContext<Self>) -> View {
            let focus = self.focus.clone();
            cx.use_effect("focus", (), move || {
                let _ = focus.request_focus();
                None
            });
            let mut children = Vec::<View>::new();
            if self.phase > 0 {
                let refs = self.refs.clone();
                let sources = self.sources.clone();
                cx.use_effect("sources", (), move || {
                    for (reference, source) in refs.iter().zip(sources) {
                        assert!(
                            reference.request_set_native_source(
                                Some(source.cast().unwrap()),
                                |result| { result.unwrap() }
                            )
                        );
                    }
                    None
                });
                children.push(
                    Image::new()
                        .source_svg(self.reactor.clone())
                        .width(192.0)
                        .height(192.0)
                        .into(),
                );
                children.extend(self.refs.iter().enumerate().map(|(index, reference)| {
                    let size = if index == 8 { 3072.0 } else { 192.0 };
                    let image = Image::new().element_ref(reference).width(size).height(size);
                    if index == 8 {
                        Canvas::new()
                            .width(192.0)
                            .height(192.0)
                            .children((image,))
                            .into()
                    } else {
                        image.into()
                    }
                }));
            }
            let cells = children
                .into_iter()
                .enumerate()
                .map(|(index, image)| {
                    let label = [
                        "Reactor auto",
                        "Native auto",
                        "Raster 24",
                        "Raster 96",
                        "Raster 192",
                        "Raster 384",
                        "Raster 768",
                        "Raster 1536",
                        "Raster 3072",
                        "Raster 3072 at 1:1",
                    ][index];
                    StackPanel::new()
                        .grid_row((index / 3) as i32)
                        .grid_column((index % 3) as i32)
                        .children((
                            TextBlock::new()
                                .text(label)
                                .foreground(Color::rgb(16, 16, 16)),
                            image,
                        ))
                        .into()
                })
                .collect::<Vec<View>>();
            cx.window_frame(
                "SVG raster size measurement",
                StackPanel::new().children((
                    Button::new()
                        .element_ref(&self.focus)
                        .content("Raster measurement"),
                    Grid::new()
                        .width(624.0)
                        .height(960.0)
                        .columns([GridLength::Star(1.0); 3])
                        .rows([GridLength::Star(1.0); 4])
                        .children(cells),
                )),
            )
        }
    }

    pub fn run() {
        App::run_component::<Experiment>(()).unwrap();
    }
}

fn main() {
    #[cfg(native_probe)]
    experiment::run();
    #[cfg(not(native_probe))]
    eprintln!("Build with WINUI_WINMD to enable the native SVG experiment");
}
