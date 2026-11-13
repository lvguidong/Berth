#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    if std::env::args().any(|a| a == "--supervised") {
        berth_lib::run();
        return;
    }
    let Ok(exe) = std::env::current_exe() else {
        berth_lib::run();
        return;
    };
    // A tiny parent waits without polling. Normal tray exits and duplicate instances
    // exit successfully; only abnormal exits trigger bounded recovery.
    let mut crashes = 0;
    loop {
        let started = std::time::Instant::now();
        let mut command = std::process::Command::new(&exe);
        command.arg("--supervised");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        match command.status() {
            Ok(status) if status.success() => break,
            Ok(_) => {
                if started.elapsed() > std::time::Duration::from_secs(60) {
                    crashes = 0;
                }
                crashes += 1;
                if crashes > 3 {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            Err(_) => {
                berth_lib::run();
                break;
            }
        }
    }
}
