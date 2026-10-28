use windows_reactor::*;

const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="1" y="1" width="22" height="22" rx="3" fill="none" stroke="#101010" stroke-width="1"/><circle cx="12" cy="12" r="7" fill="none" stroke="#101010" stroke-width=".7"/><path d="M3 21L21 3M4 6h16M4 9h16M4 15h16M4 18h16" fill="none" stroke="#101010" stroke-width=".35"/></svg>"##;

struct Probe {
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

    fn create(_: &(), _: &ComponentContext<Self>) -> Self {
        Self {
            shared: EncodedImage::new(SVG.to_vec()),
            fresh: (0..4).map(|_| EncodedImage::new(SVG.to_vec())).collect(),
        }
    }

    fn update(&mut self, _: (), _: &ComponentContext<Self>) {}

    fn view(&self, _: &(), cx: &mut ViewContext<Self>) -> View {
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
        cx.window_frame("SVG size reproduction", content)
    }
}

fn main() {
    App::run_component::<Probe>(()).unwrap();
}
