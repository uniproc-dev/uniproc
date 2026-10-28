# SVG loaded from memory has jagged edges compared to the same SVG loaded from a file

In windows-reactor, loading an SVG through `source_svg(EncodedImage)` produces visibly jagged edges, while loading identical SVG bytes through `source_file` gives smoother edges.

The reproduction displays both images at 192×192, mounted together after one second. It writes the embedded SVG beside the executable, so no separate SVG asset is needed. Compare the circle and diagonal: From file is smoother than From memory.

Expected: identical SVG data rendered at the same size should have the same edge quality regardless of whether it is loaded from a file or memory.

```rust
use std::path::PathBuf;
use std::time::Duration;
use windows_reactor::*;

const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="1" y="1" width="22" height="22" rx="3" fill="none" stroke="#101010" stroke-width="1"/><circle cx="12" cy="12" r="7" fill="none" stroke="#101010" stroke-width=".7"/><path d="M3 21L21 3M4 6h16M4 9h16M4 15h16M4 18h16" fill="none" stroke="#101010" stroke-width=".35"/></svg>"##;

struct Repro {
    path: PathBuf,
    bytes: EncodedImage,
    ready: bool,
    _timer: ComponentTimer,
}

impl Component for Repro {
    type Input = ();
    type Message = ();

    fn create(_: &(), cx: &ComponentContext<Self>) -> Self {
        let path = std::env::current_exe()
            .unwrap()
            .with_file_name("svg-raster-repro.svg");
        std::fs::write(&path, SVG).unwrap();
        Self {
            path,
            bytes: EncodedImage::new(SVG.to_vec()),
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
                            .source_file(&self.path)
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
```
