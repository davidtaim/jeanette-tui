use std::io;

use crate::nmcli_wrapper::NmcliWrapper;
mod app;
mod nmcli_wrapper;

// fn main() -> io::Result<()> {
//     let terminal = ratatui::init();
//     let app_result = app::JeanetteApp::new().run(terminal);
//     ratatui::restore();
//     app_result
// }

fn main() {
    // NmcliWrapper::rescan_networks();
    // NmcliWrapper::connect_to_network("Ditto", "DittoDittoDitto260");
    NmcliWrapper::get_saved_networks();
}
