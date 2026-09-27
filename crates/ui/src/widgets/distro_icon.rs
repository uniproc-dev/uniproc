use guicons::icon;
use windows_reactor::View;

use crate::theme::size;

pub fn distro_icon(name: &str) -> View {
    let name = name.to_ascii_lowercase();
    let icon = if name.contains("ubuntu") {
        icon!(ubuntu)
    } else if name.contains("centos") {
        icon!(centos)
    } else if name.contains("debian") {
        icon!(debian)
    } else if name.contains("fedora") {
        icon!(fedora)
    } else if name.contains("docker") {
        icon!(docker)
    } else {
        icon!(linux)
    };
    icon.size(size::Icon).build_element()
}
