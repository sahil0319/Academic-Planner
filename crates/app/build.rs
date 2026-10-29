fn main() {
    // R-D7: fluent on every OS. Not `fluent-light`: that style fixes the color scheme to light at
    // compile time, so System appearance could not follow the OS (§6 "Appearance:"). Rust sets
    // Palette.color-scheme from the saved theme instead.
    let config = slint_build::CompilerConfiguration::new().with_style("fluent".into());
    slint_build::compile_with_config("ui/app-window.slint", config)
        .expect("failed to compile Slint UI");
}
