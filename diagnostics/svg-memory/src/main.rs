use windows_reactor::*;

struct Probe {
    sources: Vec<EncodedImage>,
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

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self { sources: sources() }
    }

    fn view(&self, _: &(), cx: &mut ViewContext<Self>) -> View {
        let content = StackPanel::new().children(
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
        );
        cx.window_frame("SVG memory reproduction", content)
    }
}

fn main() {
    App::run_component::<Probe>(()).unwrap();
}
