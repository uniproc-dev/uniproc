fn main() {
    windows_reactor_setup::as_self_contained();
    windows_bindgen::builder()
        .input(std::env::var_os("WINUI_WINMD").unwrap())
        .input_default()
        .output(std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("bindings.rs"))
        .flat()
        .minimal()
        .filter_file("bindings.txt")
        .write();
}
