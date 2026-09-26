use super::bitmap::RgbaImage;

const VISIBLE_ALPHA: u8 = 24;
const MARGIN: u32 = 1;

struct Bounds {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
}

fn content_bounds(image: &RgbaImage) -> Option<Bounds> {
    let mut bounds: Option<Bounds> = None;
    for y in 0..image.height {
        for x in 0..image.width {
            let alpha = image.pixels[((y * image.width + x) * 4 + 3) as usize];
            if alpha < VISIBLE_ALPHA {
                continue;
            }
            let b = bounds.get_or_insert(Bounds {
                left: x,
                top: y,
                right: x,
                bottom: y,
            });
            b.left = b.left.min(x);
            b.top = b.top.min(y);
            b.right = b.right.max(x);
            b.bottom = b.bottom.max(y);
        }
    }
    bounds
}

fn pixel(image: &RgbaImage, x: i64, y: i64) -> [f32; 4] {
    if x < 0 || y < 0 || x >= image.width as i64 || y >= image.height as i64 {
        return [0.0; 4];
    }
    let at = ((y as u32 * image.width + x as u32) * 4) as usize;
    let p = &image.pixels[at..at + 4];
    [p[0] as f32, p[1] as f32, p[2] as f32, p[3] as f32]
}

fn overlap(a0: f32, a1: f32, cell: i64) -> f32 {
    let lo = a0.max(cell as f32);
    let hi = a1.min(cell as f32 + 1.0);
    (hi - lo).max(0.0)
}

fn resample(image: &RgbaImage, origin: (f32, f32), side: f32, size: u32, inset: u32) -> RgbaImage {
    let mut pixels = vec![0u8; (size * size * 4) as usize];
    let inner = (size - 2 * inset) as f32;
    let scale = side / inner;

    for ty in 0..size - 2 * inset {
        let sy0 = origin.1 + ty as f32 * scale;
        let sy1 = sy0 + scale;
        for tx in 0..size - 2 * inset {
            let sx0 = origin.0 + tx as f32 * scale;
            let sx1 = sx0 + scale;

            let mut sum = [0.0f32; 4];
            let mut weight = 0.0f32;
            for sy in sy0.floor() as i64..sy1.ceil() as i64 {
                let wy = overlap(sy0, sy1, sy);
                for sx in sx0.floor() as i64..sx1.ceil() as i64 {
                    let w = wy * overlap(sx0, sx1, sx);
                    let [r, g, b, a] = pixel(image, sx, sy);
                    let premultiplied = a / 255.0 * w;
                    sum[0] += r * premultiplied;
                    sum[1] += g * premultiplied;
                    sum[2] += b * premultiplied;
                    sum[3] += a * w;
                    weight += w;
                }
            }

            let at = (((ty + inset) * size + tx + inset) * 4) as usize;
            let alpha = if weight > 0.0 { sum[3] / weight } else { 0.0 };
            let coverage = alpha / 255.0 * weight;
            if coverage > 0.0 {
                pixels[at] = (sum[0] / coverage).round().clamp(0.0, 255.0) as u8;
                pixels[at + 1] = (sum[1] / coverage).round().clamp(0.0, 255.0) as u8;
                pixels[at + 2] = (sum[2] / coverage).round().clamp(0.0, 255.0) as u8;
            }
            pixels[at + 3] = alpha.round().clamp(0.0, 255.0) as u8;
        }
    }

    RgbaImage {
        pixels,
        width: size,
        height: size,
    }
}

pub fn fit_content(image: &RgbaImage, size: u32) -> RgbaImage {
    let Some(b) = content_bounds(image) else {
        return resample(image, (0.0, 0.0), image.width.max(image.height) as f32, size, 0);
    };
    let width = (b.right - b.left + 1) as f32;
    let height = (b.bottom - b.top + 1) as f32;
    let side = width.max(height);
    let origin = (
        b.left as f32 - (side - width) / 2.0,
        b.top as f32 - (side - height) / 2.0,
    );
    resample(image, origin, side, size, MARGIN)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canvas(side: u32, square: (u32, u32, u32)) -> RgbaImage {
        let (left, top, extent) = square;
        let mut pixels = vec![0u8; (side * side * 4) as usize];
        for y in top..top + extent {
            for x in left..left + extent {
                let at = ((y * side + x) * 4) as usize;
                pixels[at..at + 4].copy_from_slice(&[200, 100, 50, 255]);
            }
        }
        RgbaImage {
            pixels,
            width: side,
            height: side,
        }
    }

    fn alpha(image: &RgbaImage, x: u32, y: u32) -> u8 {
        image.pixels[((y * image.width + x) * 4 + 3) as usize]
    }

    #[test]
    fn padded_content_is_stretched_to_the_margin() {
        let fitted = fit_content(&canvas(96, (28, 28, 40)), 32);

        assert_eq!((fitted.width, fitted.height), (32, 32));
        assert_eq!(alpha(&fitted, 0, 16), 0);
        assert_eq!(alpha(&fitted, 1, 16), 255);
        assert_eq!(alpha(&fitted, 30, 16), 255);
        assert_eq!(alpha(&fitted, 31, 16), 0);
    }

    #[test]
    fn opaque_colour_survives_resampling() {
        let fitted = fit_content(&canvas(96, (28, 28, 40)), 32);
        let at = ((16 * 32 + 16) * 4) as usize;

        assert_eq!(&fitted.pixels[at..at + 4], &[200, 100, 50, 255]);
    }

    #[test]
    fn an_empty_image_stays_transparent() {
        let empty = RgbaImage {
            pixels: vec![0u8; 96 * 96 * 4],
            width: 96,
            height: 96,
        };
        let fitted = fit_content(&empty, 32);

        assert!(fitted.pixels.iter().all(|&byte| byte == 0));
    }
}
