#![no_std]
extern crate alloc;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

mod silksong_memory;
mod unstable;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use asr::{
    future::{next_tick, retry},
    settings::Gui,
    timer::TimerState,
    Process,
};

use crate::silksong_memory::attach_silksong;

asr::async_main!(stable);
asr::panic_handler!();

struct AutoSplitterState {
    timer_state: TimerState,
    segments_splitted: Option<Vec<bool>>,
    events: String,
}

impl AutoSplitterState {
    fn new() -> AutoSplitterState {
        let timer_state = asr::timer::state();
        let segments_splitted = unstable::maybe_timer_current_attempt_segments_splitted();
        AutoSplitterState {
            timer_state,
            segments_splitted,
            events: "".to_string(),
        }
    }

    fn update(&mut self) {
        let new_state = asr::timer::state();
        let new_segments = unstable::maybe_timer_current_attempt_segments_splitted();
        if new_state == self.timer_state && new_segments == self.segments_splitted {
            return;
        }

        match new_state {
            TimerState::NotRunning => {
                self.events += "3";
                asr::timer::set_variable("events", &self.events);
                asr::timer::set_variable("last", "Reset");
            }
            TimerState::Running if is_timer_state_between_runs(self.timer_state) => {
                self.events += "1";
                asr::timer::set_variable("events", &self.events);
                asr::timer::set_variable("last", "Start");
            }
            TimerState::Paused if self.timer_state == TimerState::Running => {
                self.events += "5";
                asr::timer::set_variable("events", &self.events);
                asr::timer::set_variable("last", "Pause");
            }
            TimerState::Running if self.timer_state == TimerState::Paused => {
                self.events += "5";
                asr::timer::set_variable("events", &self.events);
                asr::timer::set_variable("last", "Resume");
            }
            TimerState::Ended => {
                self.events += "1";
                asr::timer::set_variable("events", &self.events);
                asr::timer::set_variable("last", "End");
            }
            _ => {
                if let (Some(new_segments), Some(old_segments)) = (new_segments, &self.segments_splitted) {
                    if new_segments.len() < old_segments.len() {
                        self.events += "8";
                        asr::timer::set_variable("events", &self.events);
                        asr::timer::set_variable("last", "Undo");
                    } else if new_segments.len() > old_segments.len() {
                        if new_segments[new_segments.len() - 1] {
                            self.events += "1";
                            asr::timer::set_variable("events", &self.events);
                            asr::timer::set_variable("last", "Split");
                        } else {
                            self.events += "2";
                            asr::timer::set_variable("events", &self.events);
                            asr::timer::set_variable("last", "Skip");
                        }
                    }
                }
            }
        }
    }
}

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

    let mut state = AutoSplitterState::new();

    loop {
        // TODO: replace this placeholder with the actual executables
        // for each operating system / platform once the game releases.
        let process = wait_attach_silksong(&mut settings, &mut state).await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                loop {
                    settings.update();
                    state.update();

                    // TODO: Do something on every tick.
                    next_tick().await;
                }
            })
            .await;
    }
}

async fn wait_attach_silksong(
    gui: &mut Settings,
    state: &mut AutoSplitterState,
) -> Process {
    retry(|| {
        gui.update();
        state.update();
        attach_silksong()
    })
    .await
}

// --------------------------------------------------------

pub fn is_timer_state_between_runs(s: TimerState) -> bool {
    s == TimerState::NotRunning || s == TimerState::Ended
}
