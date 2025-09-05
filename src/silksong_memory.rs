use asr::Process;

// --------------------------------------------------------

static SILKSONG_NAMES: [&str; 2] = [
    "Hollow Knight Silksong.exe", // Windows
    "Hollow Knight Silksong",     // Mac, Linux
];

// --------------------------------------------------------

pub fn attach_silksong() -> Option<Process> {
    SILKSONG_NAMES.into_iter().find_map(Process::attach)
}

// --------------------------------------------------------
