use std::time::Duration;
use windows_reactor::*;

const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="1" y="1" width="22" height="22" rx="3" fill="none" stroke="#101010" stroke-width="1"/><circle cx="12" cy="12" r="7" fill="none" stroke="#101010" stroke-width=".7"/><path d="M3 21L21 3M4 6h16M4 9h16M4 15h16M4 18h16" fill="none" stroke="#101010" stroke-width=".35"/></svg>"##;

struct Probe {
    shared: EncodedImage,
    fresh: Vec<EncodedImage>,
    after_update: Vec<EncodedImage>,
    after_delay: Vec<EncodedImage>,
    _timer: ComponentTimer,
}

enum Message {
    Update,
    AfterDelay,
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
    type Message = Message;

    fn create(_: &(), cx: &ComponentContext<Self>) -> Self {
        let _ = cx.sender().send(Message::Update);
        Self {
            shared: EncodedImage::new(SVG.to_vec()),
            fresh: (0..4).map(|_| EncodedImage::new(SVG.to_vec())).collect(),
            after_update: Vec::new(),
            after_delay: Vec::new(),
            _timer: cx.set_timeout(Duration::from_secs(5), Message::AfterDelay),
        }
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        let sources = (0..4).map(|_| EncodedImage::new(SVG.to_vec())).collect();
        match message {
            Message::Update => self.after_update = sources,
            Message::AfterDelay => self.after_delay = sources,
        }
    }

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
                if let Some(source) = self.after_update.get(i) {
                    cells.push(
                        StackPanel::new()
                            .grid_row(2)
                            .grid_column(i as i32)
                            .children((
                                TextBlock::new()
                                    .text(format!("After update {size}"))
                                    .foreground(Color::rgb(16, 16, 16)),
                                image(source, *size),
                            ))
                            .into(),
                    );
                }
                if let Some(source) = self.after_delay.get(i) {
                    cells.push(
                        StackPanel::new()
                            .grid_row(3)
                            .grid_column(i as i32)
                            .children((
                                TextBlock::new()
                                    .text(format!("After 5 seconds {size}"))
                                    .foreground(Color::rgb(16, 16, 16)),
                                image(source, *size),
                            ))
                            .into(),
                    );
                }
            }
            Grid::new()
                .width(960.0)
                .height(900.0)
                .columns([GridLength::Star(1.0); 4])
                .rows([GridLength::Star(1.0); 4])
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
