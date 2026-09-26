use std::io;

pub fn encode_png(width: u32, height: u32, pixels_rgba: &[u8]) -> io::Result<Vec<u8>> {
    let expected_len = (width as usize) * (height as usize) * 4;
    if pixels_rgba.len() != expected_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "context: pixel buffer is {} bytes, expected {expected_len} for {width}x{height}",
                pixels_rgba.len()
            ),
        ));
    }

    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().map_err(io::Error::other)?;
    writer
        .write_image_data(pixels_rgba)
        .map_err(io::Error::other)?;
    writer.finish().map_err(io::Error::other)?;
    Ok(bytes)
}
