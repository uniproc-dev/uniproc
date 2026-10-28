use std::time::Duration;
use windows_reactor::*;

const SVG_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/drawing.svg");

struct Repro {
    bytes: EncodedImage,
    ready: bool,
    _timer: ComponentTimer,
}

impl Component for Repro {
    type Input = ();
    type Message = ();

    fn create(_: &(), cx: &ComponentContext<Self>) -> Self {
        Self {
            bytes: EncodedImage::new(std::fs::read(SVG_PATH).unwrap()),
            ready: false,
            _timer: cx.set_timeout(Duration::from_secs(1), ()),
        }
    }

    fn update(&mut self, _: (), _: &ComponentContext<Self>) {
        self.ready = true;
    }

    fn view(&self, _: &(), cx: &mut ViewContext<Self>) -> View {
        let content: View = if self.ready {
            Grid::new()
                .columns([GridLength::Star(1.0); 2])
                .children((
                    StackPanel::new().grid_column(0).children((
                        TextBlock::new()
                            .text("From file")
                            .foreground(Color::rgb(16, 16, 16)),
                        Image::new()
                            .source_file(SVG_PATH)
                            .unwrap()
                            .width(192.0)
                            .height(192.0),
                    )),
                    StackPanel::new().grid_column(1).children((
                        TextBlock::new()
                            .text("From memory")
                            .foreground(Color::rgb(16, 16, 16)),
                        Image::new()
                            .source_svg(self.bytes.clone())
                            .width(192.0)
                            .height(192.0),
                    )),
                ))
                .into()
        } else {
            TextBlock::new()
                .text("Loading comparison...")
                .foreground(Color::rgb(16, 16, 16))
                .into()
        };
        cx.window_frame(
            "SVG from file vs memory",
            Grid::new()
                .width(416.0)
                .height(240.0)
                .background(Color::rgb(255, 255, 255))
                .children((content,)),
        )
    }
}

fn main() {
    App::run_component::<Repro>(()).unwrap();
}
