use std::fs;
use std::path::{Path, PathBuf};

use windows_core::{HSTRING, PWSTR};

use crate::bindings::{GetPackagesByPackageFamily, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS};

use super::appx::{appx_tile_rgba, TILE_SOURCE_SIZE};
use super::bitmap::RgbaImage;
use super::trim::fit_content;
use crate::ICON_SIZE;
use crate::encode::encode_png;

const BLESS: &str = "UNIPROC_BLESS_ICONS";
const PREVIEW_SCALE: u32 = 8;
const PACKAGES: &[(&str, &str)] = &[
    ("paint", "Microsoft.Paint_8wekyb3d8bbwe"),
    ("winui-gallery", "Microsoft.WinUI3ControlsGallery_8wekyb3d8bbwe"),
    ("terminal", "Microsoft.WindowsTerminal_8wekyb3d8bbwe"),
    ("files", "Files_1y0xx7n9077q4"),
];

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

fn package_full_name(family: &str) -> Option<String> {
    let family = HSTRING::from(family);
    let mut count = 0u32;
    let mut length = 0u32;
    let sized = unsafe { GetPackagesByPackageFamily(&family, &mut count, None, &mut length, None) };
    if sized != ERROR_INSUFFICIENT_BUFFER || count == 0 {
        return None;
    }

    let mut names = vec![PWSTR::null(); count as usize];
    let mut buffer = vec![0u16; length as usize];
    let filled = unsafe {
        GetPackagesByPackageFamily(
            &family,
            &mut count,
            Some(names.as_mut_ptr()),
            &mut length,
            Some(buffer.as_mut_ptr()),
        )
    };
    if filled != ERROR_SUCCESS {
        return None;
    }
    unsafe { names.first()?.to_string().ok() }
}

#[test]
fn fitted_tiles_match_their_goldens() {
    let dir = fixtures();
    let bless = std::env::var_os(BLESS).is_some();
    let mut failures = Vec::new();

    for (name, _) in PACKAGES {
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
#[ignore = "reads the installed packages; run by hand to refresh the tile fixtures"]
fn capture_tiles_from_installed_packages() {
    let dir = fixtures();
    fs::create_dir_all(&dir).expect("create fixture dir");

    for (name, family) in PACKAGES {
        let full_name = package_full_name(family)
            .unwrap_or_else(|| panic!("{family} is not installed"));
        let tile = appx_tile_rgba(&full_name, TILE_SOURCE_SIZE)
            .unwrap_or_else(|| panic!("no tile for {full_name}"));
        fs::write(dir.join(format!("{name}.tile.png")), png_of(&tile)).expect("write tile");
        println!("{name}: {full_name}");
    }
}
