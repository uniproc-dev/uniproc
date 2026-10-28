fn main() {
    windows_reactor_setup::as_self_contained();
    println!("cargo:rerun-if-env-changed=WINUI_WINMD");
    println!("cargo:rerun-if-changed=bindings.txt");
    if let Some(metadata) = std::env::var_os("WINUI_WINMD") {
        windows_bindgen::builder()
            .input(metadata)
            .input_default()
            .output(
                std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("bindings.rs"),
            )
            .flat()
            .minimal()
            .filter_file("bindings.txt")
            .write();
        println!("cargo:rustc-cfg=native_probe");
    }
    println!("cargo:rustc-check-cfg=cfg(native_probe)");
}
