#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum SplitterAction {
    #[default]
    Pass,
    Split,
    Skip,
    Reset,
    ManualSplit,
}

impl SplitterAction {
    pub fn or_else<F: FnOnce() -> SplitterAction>(self, f: F) -> SplitterAction {
        match self {
            SplitterAction::Pass => f(),
            a => a,
        }
    }
}

pub fn should_split(b: bool) -> SplitterAction {
    if b {
        SplitterAction::Split
    } else {
        SplitterAction::Pass
    }
}
