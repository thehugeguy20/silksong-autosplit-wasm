use alloc::{boxed::Box, string::String, vec::Vec};
use asr::{
    future::next_tick,
    game_engine::unity::mono::{self, UnityPointer},
    Address64, Process,
};
use bytemuck::CheckedBitPattern;

// --------------------------------------------------------

static SILKSONG_NAMES: [&str; 2] = [
    "Hollow Knight Silksong.exe", // Windows
    "Hollow Knight Silksong",     // Mac, Linux
];

// const PRE_MENU_INTRO: &str = "Pre_Menu_Intro";
pub const MENU_TITLE: &str = "Menu_Title";
pub const QUIT_TO_MENU: &str = "Quit_To_Menu";

// static NON_PLAY_SCENES: [&str; 3] = [PRE_MENU_INTRO, MENU_TITLE, QUIT_TO_MENU];

/*
static BAD_SCENE_NAMES: [&str; 11] = [
    "Untagged",
    "left1",
    "oncomplete",
    "Attack Range",
    "onstart",
    "position",
    "looptype",
    "integer1",
    "gameObject",
    "eventTarget",
    "material",
];
*/

pub struct StringListOffsets {
    string_len: u64,
    string_contents: u64,
    /*
    list_array: u64,
    array_len: u64,
    array_contents: u64,
    */
}

impl StringListOffsets {
    fn new() -> StringListOffsets {
        StringListOffsets {
            string_len: 0x10,
            string_contents: 0x14,
            /*
            list_array: 0x10,
            array_len: 0x18,
            array_contents: 0x20,
            */
        }
    }
}

// --------------------------------------------------------

pub fn attach_silksong() -> Option<Process> {
    SILKSONG_NAMES.into_iter().find_map(Process::attach)
}

// --------------------------------------------------------

macro_rules! declare_pointers {
    ( $g:ident { $( $f:ident : $t:ty = $e:expr ),*, } ) => {
        pub struct $g {
            $( pub $f : $t ),*,
        }

        impl $g {
            pub fn new() -> $g {
                $g {
                    $( $f : $e ),*,
                }
            }
        }
    };
}

declare_pointers!(GameManagerPointers {
    scene_name: UnityPointer<2> = UnityPointer::new("GameManager", 0, &["_instance", "sceneName"]),
    next_scene_name: UnityPointer<2> = UnityPointer::new("GameManager", 0, &["_instance", "nextSceneName"]),
    entry_gate_name: UnityPointer<2> = UnityPointer::new("GameManager", 0, &["_instance", "entryGateName"]),
    game_state: UnityPointer<2> = UnityPointer::new("GameManager", 0, &["_instance", "<GameState>k__BackingField"]),
    ui_state_vanilla: UnityPointer<3> = UnityPointer::new(
        "GameManager",
        0,
        &["_instance", "<ui>k__BackingField", "uiState"],
    ),
    accepting_input: UnityPointer<3> = UnityPointer::new(
        "GameManager",
        0,
        &[
            "_instance",
            "<inputHandler>k__BackingField",
            "acceptingInput",
        ],
    ),
    hazard_death: UnityPointer<4> = UnityPointer::new(
        "GameManager",
        0,
        &[
            "_instance",
            "<hero_ctrl>k__BackingField",
            "cState",
            "hazardDeath",
        ],
    ),
    hazard_respawning: UnityPointer<4> = UnityPointer::new(
        "GameManager",
        0,
        &[
            "_instance",
            "<hero_ctrl>k__BackingField",
            "cState",
            "hazardRespawning",
        ],
    ),
    hero_recoil_frozen: UnityPointer<4> = UnityPointer::new(
        "GameManager",
        0,
        &[
            "_instance",
            "<hero_ctrl>k__BackingField",
            "cState",
            "recoilFrozen",
        ],
    ),
    hero_transition_state: UnityPointer<3> = UnityPointer::new(
        "GameManager",
        0,
        &["_instance", "<hero_ctrl>k__BackingField", "transitionState"],
    ),
});

declare_pointers!(PlayerDataPointers {
    health: UnityPointer<3> = UnityPointer::new("GameManager", 0, &["_instance", "playerData", "health"]),
});

// --------------------------------------------------------

pub struct Memory<'a> {
    pub process: &'a Process,
    pub module: Box<mono::Module>,
    pub image: mono::Image,
    pub string_list_offsets: Box<StringListOffsets>,
}

impl Memory<'_> {
    pub async fn wait_attach(process: &Process) -> Memory {
        asr::print_message("Memory wait_attach: Module wait_attach...");
        next_tick().await;
        let mut found_module = false;
        let mut needed_retry = false;
        loop {
            let module = mono::Module::wait_attach(&process, mono::Version::V3).await;
            if !found_module {
                found_module = true;
                asr::print_message("Memory wait_attach: module get_default_image...");
                next_tick().await;
            }
            for _ in 0..0x10 {
                if let Some(image) = module.get_default_image(&process) {
                    asr::print_message("Memory wait_attach: got module and image");
                    next_tick().await;
                    return Memory {
                        process,
                        module: Box::new(module),
                        image,
                        string_list_offsets: Box::new(StringListOffsets::new()),
                    };
                }
                next_tick().await;
            }
            if !needed_retry {
                needed_retry = true;
                asr::print_message("Memory wait_attach: retry...");
                next_tick().await;
            }
        }
    }

    pub fn deref<T: CheckedBitPattern, const CAP: usize>(
        &self,
        p: &UnityPointer<CAP>,
    ) -> Result<T, asr::Error> {
        p.deref(&self.process, &self.module, &self.image)
    }

    pub fn read_string<const CAP: usize>(&self, p: &UnityPointer<CAP>) -> Option<String> {
        let a: Address64 = self.deref(p).ok()?;
        let n: u32 = self
            .process
            .read(a + self.string_list_offsets.string_len)
            .ok()?;
        if !(n < 2048) {
            return None;
        }
        let w: Vec<u16> = self
            .process
            .read_vec(a + self.string_list_offsets.string_contents, n as usize)
            .ok()?;
        String::from_utf16(&w).ok()
    }
}

// --------------------------------------------------------
