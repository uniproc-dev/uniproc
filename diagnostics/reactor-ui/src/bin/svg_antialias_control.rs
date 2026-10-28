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
                refs: (0..3).map(|_| ElementRef::new()).collect(),
                reactor: EncodedImage::new(SVG.to_vec()),
                phase: 0,
                timer: Some(cx.set_timeout(Duration::from_secs(1), ())),
            }
        }

        fn update(&mut self, _: (), cx: &ComponentContext<Self>) {
            self.phase += 1;
            if self.phase == 1 {
                self.sources = vec![
                    automatic().unwrap(),
                    reactor_ui_repros::bounded_svg(SVG, 192.0, 192.0).unwrap(),
                    reactor_ui_repros::bounded_svg(SVG, 384.0, 384.0).unwrap(),
                ];
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
            let mut children: Vec<View> = vec![
                Button::new()
                    .element_ref(&self.focus)
                    .content("SVG antialias experiment")
                    .into(),
            ];
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
                children.extend(self.refs.iter().map(|reference| {
                    Image::new()
                        .element_ref(reference)
                        .width(192.0)
                        .height(192.0)
                        .into()
                }));
            }
            cx.window_frame(
                "SVG antialias experiment",
                StackPanel::new().children(children),
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
