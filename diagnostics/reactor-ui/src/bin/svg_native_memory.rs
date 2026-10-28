#[cfg(native_probe)]
mod experiment {
    use reactor_ui_repros::bindings as c;
    use std::time::Duration;
    use windows_core::{Interface, Result};
    use windows_reactor::*;

    const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#ffffff"/></svg>"##;

    fn native(index: usize) -> Result<c::SvgImageSource> {
        let source: c::SvgImageSource =
            windows_core::factory::<c::SvgImageSource, windows_core::imp::IGenericFactory>()?
                .ActivateInstance()?;
        if let Some(size) = std::env::var("MEMORY_RASTER")
            .ok()
            .and_then(|value| value.parse::<f64>().ok())
        {
            source.SetRasterizePixelWidth(size)?;
            source.SetRasterizePixelHeight(size)?;
        }
        if std::env::var_os("MEMORY_NO_LOAD").is_none() {
            let stream: c::InMemoryRandomAccessStream = windows_core::factory::<
                c::InMemoryRandomAccessStream,
                windows_core::imp::IGenericFactory,
            >()?
            .ActivateInstance()?;
            let writer = c::DataWriter::CreateDataWriter(&stream.GetOutputStreamAt(0)?)?;
            writer.WriteBytes(SVG)?;
            let store = writer.StoreAsync()?;
            let image = source.clone();
            store.when(move |result| {
                result.unwrap();
                writer.DetachStream().unwrap();
                stream.Seek(0).unwrap();
                image
                    .SetSourceAsync(&stream)
                    .unwrap()
                    .when(move |result| eprintln!("LOAD {index} {result:?}"))
                    .unwrap();
            })?;
        }
        Ok(source)
    }

    struct Probe {
        native: Vec<c::SvgImageSource>,
        refs: Vec<ElementRef<Image>>,
        phase: u32,
        timer: Option<ComponentTimer>,
        elapsed: u32,
    }

    fn heaps() {
        unsafe {
            let count = c::GetProcessHeaps(0, std::ptr::null_mut());
            let mut heaps = vec![Default::default(); count as usize];
            let count = c::GetProcessHeaps(count, heaps.as_mut_ptr()).min(count);
            let mut allocated = 0usize;
            let mut committed = 0usize;
            let mut reserved = 0usize;
            let mut failed = 0;
            for heap in heaps.into_iter().take(count as usize) {
                let mut summary = c::HEAP_SUMMARY {
                    cb: std::mem::size_of::<c::HEAP_SUMMARY>() as u32,
                    ..Default::default()
                };
                if c::HeapSummary(heap, 0, &mut summary).0 != 0 {
                    allocated += summary.cbAllocated;
                    committed += summary.cbCommitted;
                    reserved += summary.cbReserved;
                } else {
                    failed += 1;
                }
            }
            eprintln!(
                "HEAPS allocated={allocated} committed={committed} reserved={reserved} failed={failed}"
            );
        }
    }

    impl Component for Probe {
        type Input = ();
        type Message = ();

        fn create(_: &(), cx: &ComponentContext<Self>) -> Self {
            Self {
                native: Vec::new(),
                refs: (0..27).map(|_| ElementRef::new()).collect(),
                phase: 0,
                timer: Some(cx.set_timeout(Duration::from_secs(1), ())),
                elapsed: 0,
            }
        }

        fn update(&mut self, _: (), cx: &ComponentContext<Self>) {
            self.elapsed += 1;
            let cycles = std::env::var("MEMORY_CYCLES")
                .ok()
                .and_then(|value| value.parse::<u32>().ok())
                .unwrap_or(2);
            let phase = if self.elapsed < 5 {
                0
            } else {
                ((self.elapsed - 5) / 10 + 1).min(cycles * 2)
            };
            if phase != self.phase {
                self.phase = phase;
                eprintln!("PHASE {} at={:?}", self.phase, std::time::SystemTime::now());
                if self.phase % 2 == 1 && self.native.is_empty() {
                    self.native = (0..27).map(|index| native(index).unwrap()).collect();
                }
                if self.phase.is_multiple_of(2) && std::env::var_os("MEMORY_RELEASE").is_some() {
                    self.native.clear();
                }
            }
            heaps();
            if self.elapsed >= cycles * 20 + 5 {
                let _ = cx.close_window();
                self.timer = None;
            } else {
                self.timer = Some(cx.set_timeout(Duration::from_secs(1), ()));
            }
        }

        fn view(&self, _: &(), cx: &mut ViewContext<Self>) -> View {
            let visible = self.phase % 2 == 1 && std::env::var_os("MEMORY_NO_ATTACH").is_none();
            let content: View = if visible {
                let refs = self.refs.clone();
                let sources = self.native.clone();
                cx.use_effect("native-sources", self.phase, move || {
                    for (reference, source) in refs.iter().zip(sources) {
                        let _ = reference.request_set_native_source(
                            Some(source.cast().unwrap()),
                            |result| {
                                if let Err(error) = result {
                                    eprintln!("ATTACH {error:?}");
                                }
                            },
                        );
                    }
                    None
                });
                StackPanel::new()
                    .children(
                        self.refs
                            .iter()
                            .map(|reference| {
                                Image::new()
                                    .element_ref(reference)
                                    .width(20.0)
                                    .height(20.0)
                                    .into()
                            })
                            .collect::<Vec<View>>(),
                    )
                    .into()
            } else {
                TextBlock::new().text("Native SVG memory experiment").into()
            };
            cx.window_frame("Native SVG memory experiment", content)
        }
    }

    pub fn run() {
        App::run_component::<Probe>(()).unwrap();
    }
}

fn main() {
    #[cfg(native_probe)]
    experiment::run();
    #[cfg(not(native_probe))]
    eprintln!("Build with WINUI_WINMD to enable the native SVG experiment");
}
