#![no_std]
extern crate alloc;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

mod silksong_memory;
mod unstable;

use alloc::string::{String, ToString};
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
    split_index: Option<i32>,
    events: String,
}

impl AutoSplitterState {
    fn new() -> AutoSplitterState {
        let timer_state = asr::timer::state();
        let split_index = unstable::maybe_timer_current_split_index();
        AutoSplitterState {
            timer_state,
            split_index,
            events: "".to_string(),
        }
    }

    fn update(&mut self) {
        let new_state = asr::timer::state();
        let new_index = unstable::maybe_timer_current_split_index();
        if new_state == self.timer_state && new_index == self.split_index {
            return;
        }

        match new_state {
            TimerState::NotRunning if self.timer_state == TimerState::Running || self.timer_state == TimerState::Paused || self.timer_state == TimerState::Ended => {
                self.events += "3";
                asr::timer::set_variable("events", str_take_right(&self.events, 10));
                asr::timer::set_variable("last", "Reset");
                asr::print_message("Reset.");
            }
            TimerState::Running if is_timer_state_between_runs(self.timer_state) => {
                self.events += "1";
                asr::timer::set_variable("events", str_take_right(&self.events, 10));
                asr::timer::set_variable("last", "Start");
                asr::print_message("Started.");
            }
            TimerState::Paused if self.timer_state == TimerState::Running => {
                self.events += "5";
                asr::timer::set_variable("events", str_take_right(&self.events, 10));
                asr::timer::set_variable("last", "Pause");
                asr::print_message("Paused.");
            }
            TimerState::Running if self.timer_state == TimerState::Paused => {
                self.events += "5";
                asr::timer::set_variable("events", str_take_right(&self.events, 10));
                asr::timer::set_variable("last", "Resume");
                asr::print_message("Resumed.");
            }
            TimerState::Ended if self.timer_state == TimerState::Running || self.timer_state == TimerState::Paused => {
                self.events += "1";
                asr::timer::set_variable("events", str_take_right(&self.events, 10));
                asr::timer::set_variable("last", "End");
                asr::print_message("Ended.");
            }
            _ => {
                if let (Some(new_index), Some(old_index)) = (&new_index, &self.split_index) {
                    if new_index < old_index {
                        self.events += "8";
                        asr::timer::set_variable("events", str_take_right(&self.events, 10));
                        asr::timer::set_variable("last", "Undo");
                        asr::print_message("Undid.");
                    } else if new_index > old_index {
                        if unstable::maybe_timer_segment_splitted(new_index - 1).unwrap_or_default() {
                            self.events += "1";
                            asr::timer::set_variable("events", str_take_right(&self.events, 10));
                            asr::timer::set_variable("last", "Split");
                            asr::print_message("Splitted.");
                        } else {
                            self.events += "2";
                            asr::timer::set_variable("events", str_take_right(&self.events, 10));
                            asr::timer::set_variable("last", "Skip");
                            asr::print_message("Skipped.");
                        }
                    }
                }
            }
        }

        self.timer_state = new_state;
        self.split_index = new_index;
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

pub fn str_take_right(s: &str, n: usize) -> &str {
    s.split_at(s.len().saturating_sub(n)).1
}
