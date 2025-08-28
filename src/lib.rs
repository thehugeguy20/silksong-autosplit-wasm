#![no_std]
extern crate alloc;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

use asr::{future::next_tick, settings::Gui, Process};

asr::async_main!(stable);
asr::panic_handler!();

#[derive(Gui)]
struct Settings {
    /// My Setting
    #[default = true]
    my_setting: bool,
    // TODO: Change these settings.
}

async fn main() {
    // TODO: Set up some general state and settings.
    let mut settings = Settings::register();

    asr::print_message("Hello, World!");

    loop {
        // TODO: replace this placeholder with the actual executables
        // for each operating system / platform once the game releases.
        let process = Process::wait_attach("silksong.exe").await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                loop {
                    settings.update();

                    // TODO: Do something on every tick.
                    next_tick().await;
                }
            })
            .await;
    }
}
