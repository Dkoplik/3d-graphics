pub mod app;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "ColorsApp",
        native_options,
        Box::new(|cc| Ok(Box::new(app::AthenianApp::new(cc)))),
    )
}
