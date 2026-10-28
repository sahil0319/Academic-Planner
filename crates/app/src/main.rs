slint::include_modules!();

mod viewmodel;

fn main() -> anyhow::Result<()> {
    let window = AppWindow::new()?;
    window.run()?;
    Ok(())
}
