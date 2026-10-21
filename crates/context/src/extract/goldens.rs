use std::fs;
use std::path::{Path, PathBuf};

use super::bitmap::RgbaImage;
use super::trim::fit_content;
use crate::ICON_SIZE;
use crate::encode::encode_png;

const BLESS: &str = "UNIPROC_BLESS_ICONS";
const PREVIEW_SCALE: u32 = 8;
const FIXTURES: &[&str] = &["square", "wide", "tall", "translucent"];

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures").join("icons")
}

fn mismatches() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/icon-goldens")
}

fn decode_png(path: &Path) -> RgbaImage {
    let file = fs::File::open(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().expect("png header");
    let mut pixels = vec![0u8; reader.output_buffer_size().expect("png size")];
    let info = reader.next_frame(&mut pixels).expect("png frame");
    assert_eq!(
        info.color_type,
        png::ColorType::Rgba,
        "{} must be RGBA",
        path.display()
    );
    pixels.truncate(info.buffer_size());
    RgbaImage {
        pixels,
        width: info.width,
        height: info.height,
    }
}

fn png_of(image: &RgbaImage) -> Vec<u8> {
    encode_png(image.width, image.height, &image.pixels).expect("png encode")
}

fn enlarged(image: &RgbaImage) -> RgbaImage {
    let width = image.width * PREVIEW_SCALE;
    let height = image.height * PREVIEW_SCALE;
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let at = (((y / PREVIEW_SCALE) * image.width + x / PREVIEW_SCALE) * 4) as usize;
            pixels.extend_from_slice(&image.pixels[at..at + 4]);
        }
    }
    RgbaImage {
        pixels,
        width,
        height,
    }
}

fn write_pair(dir: &Path, stem: &str, image: &RgbaImage) {
    fs::create_dir_all(dir).expect("create output dir");
    fs::write(dir.join(format!("{stem}.png")), png_of(image)).expect("write png");
    fs::write(dir.join(format!("{stem}.x8.png")), png_of(&enlarged(image))).expect("write preview");
}

#[test]
fn fitted_tiles_match_their_goldens() {
    let dir = fixtures();
    let bless = std::env::var_os(BLESS).is_some();
    let mut failures = Vec::new();

    for name in FIXTURES {
        let tile_path = dir.join(format!("{name}.tile.png"));
        if !tile_path.exists() {
            failures.push(format!("{name}: no fixture {}", tile_path.display()));
            continue;
        }
        let fitted = fit_content(&decode_png(&tile_path), ICON_SIZE as u32);

        if bless {
            write_pair(&dir, &format!("{name}.fitted"), &fitted);
            continue;
        }

        let golden_path = dir.join(format!("{name}.fitted.png"));
        let golden = fs::read(&golden_path).unwrap_or_default();
        if golden != png_of(&fitted) {
            let out = mismatches();
            write_pair(&out, &format!("{name}.actual"), &fitted);
            failures.push(format!(
                "{name}: differs from {}; actual is {}",
                golden_path.display(),
                out.join(format!("{name}.actual.x8.png")).display()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{}\nrerun with {BLESS}=1 to accept the new output",
        failures.join("\n")
    );
}

#[test]
#[ignore = "regenerates deterministic synthetic tile inputs; bless goldens separately"]
fn generate_neutral_tiles() {
    let dir = fixtures();
    fs::create_dir_all(&dir).expect("create fixture dir");

    for name in FIXTURES {
        let mut tile = RgbaImage {
            width: 96,
            height: 96,
            pixels: vec![0; 96 * 96 * 4],
        };
        let (left, top, width, height, opacity) = match *name {
            "square" => (28, 28, 40, 40, 255),
            "wide" => (12, 36, 72, 24, 255),
            "tall" => (38, 8, 20, 80, 255),
            "translucent" => (20, 24, 56, 48, 144),
            _ => unreachable!(),
        };
        for row in top..top + height {
            for column in left..left + width {
                let offset = (row * 96 + column) * 4;
                let color = if column < left + width / 2 {
                    [48, 96, 144, opacity]
                } else {
                    [176, 112, 64, opacity]
                };
                tile.pixels[offset..offset + 4].copy_from_slice(&color);
            }
        }
        fs::write(dir.join(format!("{name}.tile.png")), png_of(&tile)).expect("write tile");
    }
}
