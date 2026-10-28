fn main() {
    // R-D7: fluent-light so standard widgets render in a light theme on every OS.
    let config = slint_build::CompilerConfiguration::new().with_style("fluent-light".into());
    slint_build::compile_with_config("ui/app-window.slint", config)
        .expect("failed to compile Slint UI");
}
