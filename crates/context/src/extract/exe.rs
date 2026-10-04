use windows_core::PCWSTR;

use crate::bindings::{
    DestroyIcon, PrivateExtractIconsW, SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON,
};

use super::bitmap::{hicon_to_rgba, RgbaImage};

pub fn extract_icon_rgba(path: &str) -> Option<RgbaImage> {
    if !has_own_icon(path) {
        return None;
    }

    unsafe {
        let mut shfi: SHFILEINFOW = std::mem::zeroed();
        let path_wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

        let result = SHGetFileInfoW(
            PCWSTR(path_wide.as_ptr()),
            0,
            Some(&mut shfi),
            size_of::<SHFILEINFOW>() as u32,
            (SHGFI_ICON | SHGFI_SMALLICON) as u32,
        );

        if result == 0 || shfi.hIcon.0.is_null() {
            return None;
        }

        let img = hicon_to_rgba(shfi.hIcon);
        let _ = DestroyIcon(shfi.hIcon);
        img
    }
}

pub fn has_own_icon(exe_path: &str) -> bool {
    let mut buffer = [0u16; 260];
    for (i, wide_char) in exe_path.encode_utf16().enumerate() {
        if i >= 259 {
            break;
        }
        buffer[i] = wide_char;
    }

    unsafe { PrivateExtractIconsW(PCWSTR(buffer.as_ptr()), 0, 0, 0, None, None, 0, 0) > 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPLORER: &str = r"C:\Windows\explorer.exe";

    #[test]
    fn an_executable_with_an_icon_gives_a_visible_image() {
        assert!(has_own_icon(EXPLORER));
        let image = extract_icon_rgba(EXPLORER).expect("explorer has an icon");

        assert!(image.width > 0 && image.height > 0);
        assert_eq!(image.pixels.len(), (image.width * image.height * 4) as usize);
        assert!(image.pixels.chunks_exact(4).any(|pixel| pixel[3] > 0), "all transparent");
    }

    #[test]
    fn a_file_without_an_icon_gives_none() {
        let path = r"C:\Windows\System32\drivers\etc\hosts";
        assert!(!has_own_icon(path));
        assert!(extract_icon_rgba(path).is_none());
    }
}
