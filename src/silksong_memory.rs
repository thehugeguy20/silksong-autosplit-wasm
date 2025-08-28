use asr::Process;

// --------------------------------------------------------

// TODO: replace these placeholders with the actual executables
// for each operating system / platform once the game releases.
static SILKSONG_NAMES: [&str; 5] = [
    "silksong.exe",    // Windows
    "silksong.x86_64", // Linux
    "Silksong.exe",    // Linux GOG?
    "Silksong",        // Mac
    "silksong",        // Mac
];

// --------------------------------------------------------

pub fn attach_silksong() -> Option<Process> {
    SILKSONG_NAMES.into_iter().find_map(Process::attach)
}

// --------------------------------------------------------
