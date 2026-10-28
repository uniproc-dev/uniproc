use std::time::Duration;
use windows_reactor::*;

const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="1" y="1" width="22" height="22" rx="3" fill="none" stroke="#101010" stroke-width="1"/><circle cx="12" cy="12" r="7" fill="none" stroke="#101010" stroke-width=".7"/><path d="M3 21L21 3M4 6h16M4 9h16M4 15h16M4 18h16" fill="none" stroke="#101010" stroke-width=".35"/></svg>"##;

struct Probe {
    shared: EncodedImage,
    fresh: Vec<EncodedImage>,
    ready: bool,
    _timer: ComponentTimer,
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
            shared: EncodedImage::new(SVG.to_vec()),
            fresh: (0..4).map(|_| EncodedImage::new(SVG.to_vec())).collect(),
            ready: false,
            _timer: cx.set_timeout(Duration::from_secs(5), ()),
        }
    }

    fn update(&mut self, _: (), _: &ComponentContext<Self>) {
        self.ready = true;
    }

    fn view(&self, _: &(), cx: &mut ViewContext<Self>) -> View {
        let sizes = [24.0, 48.0, 96.0, 192.0];
        let content: View = if self.ready {
            let mut cells = vec![
                TextBlock::new()
                    .text("Shared source prewarmed at first size")
                    .foreground(Color::rgb(16, 16, 16))
                    .into(),
            ];
            for (i, size) in sizes.iter().enumerate() {
                cells.push(
                    StackPanel::new()
                        .grid_row(1)
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
                        .grid_row(2)
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
        cx.window_frame("SVG size reproduction", content)
    }
}

fn main() {
    App::run_component::<Probe>(()).unwrap();
}
