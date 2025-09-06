#![no_std]
extern crate alloc;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

mod silksong_memory;
mod unstable;

use alloc::{boxed::Box, format};
use asr::{
    future::{next_tick, retry},
    settings::Gui,
    timer::TimerState,
    Address64, Process,
};

use crate::silksong_memory::{
    attach_silksong, GameManagerPointers, Memory, PlayerDataPointers, MENU_TITLE, QUIT_TO_MENU,
};

asr::async_main!(stable);
asr::panic_handler!();

// --------------------------------------------------------

pub const GAME_STATE_INACTIVE: i32 = 0;
pub const GAME_STATE_MAIN_MENU: i32 = 1;
pub const GAME_STATE_LOADING: i32 = 2;
pub const GAME_STATE_ENTERING_LEVEL: i32 = 3;
pub const GAME_STATE_PLAYING: i32 = 4;
// pub const GAME_STATE_PAUSED: i32 = 5;
pub const GAME_STATE_EXITING_LEVEL: i32 = 6;
pub const GAME_STATE_CUTSCENE: i32 = 7;

// UI_STATE 1: Main Menu
pub const UI_STATE_PLAYING: i32 = 4;
pub const UI_STATE_PAUSED: i32 = 5;

// HERO_TRANSITION_STATE 0: N/A, not in transition
// HERO_TRANSITION_STATE 1: Exiting scene
// HERO_TRANSITION_STATE 2, 3: Waiting to enter, Entering?
pub const HERO_TRANSITION_STATE_WAITING_TO_ENTER_LEVEL: i32 = 2;

// --------------------------------------------------------

struct AutoSplitterState {
    timer_state: TimerState,
    split_index: Option<u64>,
    look_for_teleporting: bool,
    last_ui_state: i32,
    last_game_state: i32,
    last_hero_transition_state: i32,
    hits: i64,
    last_recoil: bool,
    last_hazard: bool,
    last_health_0: bool,
    last_health: Option<i32>,
    last_paused: bool,
}

impl AutoSplitterState {
    fn new() -> AutoSplitterState {
        let timer_state = asr::timer::state();
        let split_index = unstable::timer_current_split_index();
        AutoSplitterState {
            timer_state,
            split_index,
            look_for_teleporting: false,
            last_ui_state: 0,
            last_game_state: GAME_STATE_INACTIVE,
            last_hero_transition_state: 0,
            hits: 0,
            last_recoil: false,
            last_hazard: false,
            last_health_0: false,
            last_health: None,
            last_paused: false,
        }
    }

    fn update(&mut self) {
        let new_state = asr::timer::state();
        let new_index = unstable::timer_current_split_index();
        if new_state == self.timer_state && new_index == self.split_index {
            return;
        }

        match new_state {
            TimerState::NotRunning
                if self.timer_state == TimerState::Running
                    || self.timer_state == TimerState::Paused
                    || self.timer_state == TimerState::Ended =>
            {
                // Reset
            }
            TimerState::Running if is_timer_state_between_runs(self.timer_state) => {
                // Start
            }
            TimerState::Paused if self.timer_state == TimerState::Running => {
                // Pause
            }
            TimerState::Running if self.timer_state == TimerState::Paused => {
                // Resume
            }
            TimerState::Ended
                if self.timer_state == TimerState::Running
                    || self.timer_state == TimerState::Paused =>
            {
                // End
            }
            _ => {
                if let (Some(new_index), Some(old_index)) = (&new_index, &self.split_index) {
                    if new_index < old_index {
                        // Undo
                    } else if new_index > old_index {
                        if unstable::timer_segment_splitted(new_index - 1).unwrap_or_default() {
                            // Split
                        } else {
                            // Skip
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
    /// Hit Counter
    #[default = true]
    hit_counter: bool,
    // TODO: Change these settings.
}

async fn main() {
    // TODO: Set up some general state and settings.
    let mut settings = Settings::register();

    asr::print_message("Hello, World!");

    // register the variables on start
    asr::timer::set_variable_int("hits", 0);

    let mut state = AutoSplitterState::new();

    loop {
        // TODO: replace this placeholder with the actual executables
        // for each operating system / platform once the game releases.
        let process = wait_attach_silksong(&mut settings, &mut state).await;
        process
            .until_closes(async {
                // TODO: Load some initial information from the process.
                let mem = Memory::wait_attach(&process).await;
                let gm = Box::new(GameManagerPointers::new());
                let pd = Box::new(PlayerDataPointers::new());
                let _: bool = mem.deref(&gm.accepting_input).unwrap_or_default();
                let _: Address64 = mem.deref(&gm.entry_gate_name).unwrap_or_default();
                let _: i32 = mem.deref(&gm.game_state).unwrap_or_default();
                let _: bool = mem.deref(&gm.hazard_death).unwrap_or_default();
                let _: bool = mem.deref(&gm.hazard_respawning).unwrap_or_default();
                let _: bool = mem.deref(&gm.hero_recoil_frozen).unwrap_or_default();
                let _: i32 = mem.deref(&gm.hero_transition_state).unwrap_or_default();
                let _: Address64 = mem.deref(&gm.next_scene_name).unwrap_or_default();
                let _: Address64 = mem.deref(&gm.scene_name).unwrap_or_default();
                let _: i32 = mem.deref(&gm.ui_state_vanilla).unwrap_or_default();
                let _: i32 = mem.deref(&pd.health).unwrap_or_default();
                next_tick().await;
                loop {
                    settings.update();
                    state.update();

                    // TODO: Do something on every tick.
                    load_removal(&mut state, &mem, &gm);
                    handle_hits(&mut state, &mem, &gm, &pd);
                    next_tick().await;
                }
            })
            .await;
    }
}

async fn wait_attach_silksong(gui: &mut Settings, state: &mut AutoSplitterState) -> Process {
    retry(|| {
        gui.update();
        state.update();
        attach_silksong()
    })
    .await
}

// --------------------------------------------------------

fn load_removal(state: &mut AutoSplitterState, mem: &Memory, gm: &GameManagerPointers) {
    // only remove loads if timer is running
    if asr::timer::state() != TimerState::Running {
        asr::timer::pause_game_time();
        return;
    }

    let ui_state: i32 = mem.deref(&gm.ui_state_vanilla).unwrap_or_default();
    let scene_name = mem.read_string(&gm.scene_name).unwrap_or_default();
    let next_scene = mem.read_string(&gm.next_scene_name).unwrap_or_default();

    let loading_menu = (scene_name != MENU_TITLE && next_scene.is_empty())
        || (scene_name != MENU_TITLE && next_scene == MENU_TITLE || (scene_name == QUIT_TO_MENU));

    // TODO: teleporting, look_for_teleporting

    let game_state: i32 = mem.deref(&gm.game_state).unwrap_or_default();

    if game_state == GAME_STATE_PLAYING && state.last_game_state == GAME_STATE_MAIN_MENU {
        state.look_for_teleporting = true;
    }
    if state.look_for_teleporting
        && (game_state != GAME_STATE_PLAYING && game_state != GAME_STATE_ENTERING_LEVEL)
    {
        state.look_for_teleporting = false;
    }

    // TODO: hazard_respawning
    let accepting_input: bool = mem.deref(&gm.accepting_input).unwrap_or_default();
    let hero_transition_state: i32 = mem.deref(&gm.hero_transition_state).unwrap_or_default();
    // TODO: tile_map_dirty, uses_scene_transition_routine

    let is_game_time_paused = (state.look_for_teleporting)
        || ((game_state == GAME_STATE_PLAYING || game_state == GAME_STATE_ENTERING_LEVEL)
            && ui_state != UI_STATE_PLAYING)
        || (game_state != GAME_STATE_PLAYING && !accepting_input)
        || (game_state == GAME_STATE_EXITING_LEVEL || game_state == GAME_STATE_LOADING)
        || (hero_transition_state == HERO_TRANSITION_STATE_WAITING_TO_ENTER_LEVEL)
        || (ui_state != UI_STATE_PLAYING
            && (loading_menu
                || (ui_state != UI_STATE_PAUSED
                    && (!next_scene.is_empty() || scene_name == "_test_charms")))
            && next_scene != scene_name);
    if is_game_time_paused {
        asr::timer::pause_game_time();
    } else {
        asr::timer::resume_game_time();
    }

    {
        if ui_state != state.last_ui_state {
            asr::print_message(&format!("ui_state: {}", ui_state));
        }
        state.last_ui_state = ui_state;
    }

    if game_state != state.last_game_state {
        asr::print_message(&format!("game_state: {}", game_state));
    }
    state.last_game_state = game_state;

    {
        if hero_transition_state != state.last_hero_transition_state {
            asr::print_message(&format!("hero_transition_state: {}", hero_transition_state));
        }
        state.last_hero_transition_state = hero_transition_state;
    }

    {
        if is_game_time_paused != state.last_paused {
            asr::print_message(&format!("is_game_time_paused: {}", is_game_time_paused));
        }
        state.last_paused = is_game_time_paused;
    }
}

fn handle_hits(
    state: &mut AutoSplitterState,
    mem: &Memory,
    gm: &GameManagerPointers,
    pd: &PlayerDataPointers,
) {
    // only count hits if timer is running
    if asr::timer::state() != TimerState::Running {
        return;
    }

    let recoil: bool = mem.deref(&gm.hero_recoil_frozen).unwrap_or_default();
    if !state.last_recoil && recoil {
        add_hit(state);
        asr::print_message(&format!("hit: {}, from recoil", state.hits));
    }
    state.last_recoil = recoil;

    let hazard: bool = mem.deref(&gm.hazard_death).unwrap_or_default();
    if !state.last_hazard && hazard {
        add_hit(state);
        asr::print_message(&format!("hit: {}, from hazard", state.hits));
    }
    state.last_hazard = hazard;

    let maybe_health: Option<i32> = mem.deref(&pd.health).ok();
    let game_state: i32 = mem.deref(&gm.game_state).unwrap_or_default();
    let health_0 = maybe_health == Some(0) && game_state == GAME_STATE_PLAYING;
    if !state.last_health_0 && health_0 {
        add_hit(state);
        asr::print_message(&format!("hit: {}, from heath 0", state.hits));
    }
    state.last_health_0 = health_0;

    {
        if maybe_health != state.last_health {
            asr::print_message(&format!("health: {:?}", maybe_health));
        }
        state.last_health = maybe_health;
    }
}

fn add_hit(state: &mut AutoSplitterState) {
    state.hits += 1;
    asr::timer::set_variable_int("hits", state.hits);
}

// --------------------------------------------------------

pub fn is_timer_state_between_runs(s: TimerState) -> bool {
    s == TimerState::NotRunning || s == TimerState::Ended
}

pub fn str_take_right(s: &str, n: usize) -> &str {
    s.split_at(s.len().saturating_sub(n)).1
}
