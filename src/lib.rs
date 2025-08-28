#![no_std]
extern crate alloc;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

mod silksong_memory;

use asr::{
    future::{next_tick, retry},
    settings::Gui,
    Process,
};

use crate::silksong_memory::attach_silksong;

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
        let process = wait_attach_silksong(&mut settings).await;
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

async fn wait_attach_silksong(
    gui: &mut Settings,
) -> Process {
    retry(|| {
        gui.update();
        attach_silksong()
    })
    .await
}
