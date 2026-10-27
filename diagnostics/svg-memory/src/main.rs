use std::time::Duration;
use windows_reactor::*;

struct Probe {
    sources: Vec<EncodedImage>,
    step: u32,
    timer: Option<ComponentTimer>,
}

fn sources() -> Vec<EncodedImage> {
    let explicit = std::env::var_os("SVG_PIXELS").is_some();
    let dimension = if explicit { "24" } else { "1em" };
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{dimension}" height="{dimension}" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" fill="#ffffff"/></svg>"##
    );
    let count = std::env::var("SVG_COUNT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(27);
    let source = EncodedImage::new(svg.as_bytes().to_vec());
    let shared = std::env::var_os("SVG_SHARED").is_some();
    (0..count)
        .map(|_| {
            if shared {
                source.clone()
            } else {
                EncodedImage::new(svg.as_bytes().to_vec())
            }
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
        if std::env::var_os("SVG_RELEASE_ON_HIDE").is_some() {
            if self.step.is_multiple_of(2) {
                self.sources.clear();
            } else if self.sources.is_empty() {
                self.sources = sources();
            }
        }
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
