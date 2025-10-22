use std::io;
mod app;
mod nmcli_wrapper;

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let app_result = app::JeanetteApp::new().run(terminal);
    ratatui::restore();
    app_result
}
