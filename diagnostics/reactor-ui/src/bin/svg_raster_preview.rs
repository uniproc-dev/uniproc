#[cfg(native_probe)]
#[allow(
    clippy::all,
    dead_code,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals
)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(native_probe)]
mod reproduction {
    use super::bindings as c;
    use std::time::Duration;
    use windows_core::{Interface, Result};
    use windows_reactor::*;

    const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="1" y="1" width="22" height="22" rx="3" fill="none" stroke="#101010" stroke-width="1"/><circle cx="12" cy="12" r="7" fill="none" stroke="#101010" stroke-width=".7"/><path d="M3 21L21 3M4 6h16M4 9h16M4 15h16M4 18h16" fill="none" stroke="#101010" stroke-width=".35"/></svg>"##;

    fn native(raster: Option<f64>) -> Result<c::SvgImageSource> {
        let source: c::SvgImageSource =
            windows_core::factory::<c::SvgImageSource, windows_core::imp::IGenericFactory>()?
                .ActivateInstance()?;
        if let Some(size) = raster {
            source.SetRasterizePixelWidth(size)?;
            source.SetRasterizePixelHeight(size)?;
        }
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
                .when(move |result| eprintln!("raster={raster:?}: {result:?}"))
                .unwrap();
        })?;
        Ok(source)
    }

    struct Repro {
        reactor: EncodedImage,
        sources: Vec<c::SvgImageSource>,
        refs: Vec<ElementRef<Image>>,
        _timer: ComponentTimer,
    }

    impl Component for Repro {
        type Input = ();
        type Message = ();

        fn create(_: &(), cx: &ComponentContext<Self>) -> Self {
            Self {
                reactor: EncodedImage::new(SVG.to_vec()),
                sources: Vec::new(),
                refs: (0..8).map(|_| ElementRef::new()).collect(),
                _timer: cx.set_timeout(Duration::from_secs(1), ()),
            }
        }

        fn update(&mut self, _: (), _: &ComponentContext<Self>) {
            self.sources = [
                None,
                Some(24.0),
                Some(96.0),
                Some(192.0),
                Some(384.0),
                Some(768.0),
                Some(1536.0),
                Some(3072.0),
            ]
            .map(|size| native(size).unwrap())
            .into();
        }

        fn view(&self, _: &(), cx: &mut ViewContext<Self>) -> View {
            let content: View =
                if self.sources.is_empty() {
                    TextBlock::new()
                        .text("Loading SVG comparison...")
                        .foreground(Color::rgb(16, 16, 16))
                        .into()
                } else {
                    let refs = self.refs.clone();
                    let sources = self.sources.clone();
                    cx.use_effect("native-sources", (), move || {
                        for (reference, source) in refs.iter().zip(sources) {
                            assert!(reference.request_set_native_source(
                                Some(source.cast().unwrap()),
                                |result| result.unwrap()
                            ));
                        }
                        None
                    });
                    let images = std::iter::once(
                        Image::new()
                            .source_svg(self.reactor.clone())
                            .width(192.0)
                            .height(192.0)
                            .into(),
                    )
                    .chain(self.refs.iter().map(|reference| {
                        Image::new()
                            .element_ref(reference)
                            .width(192.0)
                            .height(192.0)
                            .into()
                    }));
                    let labels = [
                        "Reactor auto",
                        "Native auto",
                        "Native raster 24",
                        "Native raster 96",
                        "Native raster 192",
                        "Native raster 384",
                        "Native raster 768",
                        "Native raster 1536",
                        "Native raster 3072",
                    ];
                    Grid::new()
                        .columns([GridLength::Star(1.0); 3])
                        .rows([GridLength::Star(1.0); 3])
                        .children(
                            labels
                                .into_iter()
                                .zip(images)
                                .enumerate()
                                .map(|(index, (label, image)): (_, (_, View))| {
                                    StackPanel::new()
                                        .width(208.0)
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
                                .collect::<Vec<View>>(),
                        )
                        .into()
                };
            cx.window_frame(
                "SVG raster size comparison",
                Grid::new()
                    .width(624.0)
                    .height(720.0)
                    .background(Color::rgb(255, 255, 255))
                    .children((content,)),
            )
        }
    }

    pub fn run() {
        App::run_component::<Repro>(()).unwrap();
    }
}

fn main() {
    #[cfg(native_probe)]
    reproduction::run();
    #[cfg(not(native_probe))]
    panic!("Set WINUI_WINMD to the WinUI metadata directory before building this reproduction");
}
