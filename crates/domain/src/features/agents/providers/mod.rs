cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub mod synthetic;
        pub mod wsl;
        pub mod windows;
    } else {
        pub mod linux;
    }
}
