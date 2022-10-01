use discrakt::{
    discord::Discord,
    trakt::Trakt,
    utils::{load_config, log},
};
use std::{thread::sleep, time::Duration};

// https://stackoverflow.com/questions/29763647/how-to-make-a-program-that-does-not-display-the-console-window
fn hide_console_window() {
    use std::ptr;
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};

    let window = unsafe {GetConsoleWindow()};
    // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
    if window != ptr::null_mut() {
        unsafe {
            ShowWindow(window, SW_HIDE);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    hide_console_window();

    let cfg = load_config();
    let mut discord = Discord::new(cfg.discord_token);
    let mut trakt = Trakt::new(cfg.trakt_client_id, cfg.trakt_username);
    Discord::connect(&mut discord);

    loop {
        sleep(Duration::from_secs(15));

        let response = match Trakt::get_watching(&trakt) {
            Some(response) => response,
            None => {
                log("Nothing is being played");
                // resets the connection to also reset the activity
                Discord::close(&mut discord);
                continue;
            }
        };

        Discord::set_activity(&mut discord, &response, &mut trakt);
    }
}
