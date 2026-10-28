use std::time::Duration;
use windows_reactor::*;

struct Probe {
    sources: Vec<EncodedImage>,
    step: u32,
    timer: Option<ComponentTimer>,
}

fn sources() -> Vec<EncodedImage> {
    (0..27)
        .map(|index| {
            let color = 0x204060 + index * 0x050307;
            let svg = format!(
                r##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#{color:06x}"/></svg>"##
            );
            EncodedImage::new(svg.into_bytes())
        })
        .collect()
}

impl Component for Probe {
    type Input = ();
    type Message = ();

    fn create(_: &(), cx: &ComponentContext<Self>) -> Self {
        Self {
            sources: sources(),
            step: 0,
            timer: Some(cx.set_timeout(Duration::from_secs(5), ())),
        }
    }

    fn update(&mut self, _: (), cx: &ComponentContext<Self>) {
        self.step += 1;
        eprintln!(
            "SVG_PROBE step={} at={:?}",
            self.step,
            std::time::SystemTime::now()
        );
        self.timer = (self.step < 4).then(|| cx.set_timeout(Duration::from_secs(10), ()));
    }

    fn view(&self, _: &(), cx: &mut ViewContext<Self>) -> View {
        let content: View = if self.step % 2 == 1 {
            StackPanel::new()
                .children(
                    self.sources
                        .iter()
                        .map(|source| {
                            Image::new()
                                .width(20.0)
                                .height(20.0)
                                .source_svg(source.clone())
                                .into()
                        })
                        .collect::<Vec<View>>(),
                )
                .into()
        } else {
            TextBlock::new().text("SVG sources are unmounted").into()
        };
        cx.window_frame("SVG memory reproduction", content)
    }
}

fn main() {
    App::run_component::<Probe>(()).unwrap();
}
