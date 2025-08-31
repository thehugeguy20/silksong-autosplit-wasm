
pub fn maybe_timer_current_split_index() -> Option<i32> {
    #[cfg(feature = "unstable")]
    return Some(asr::timer::current_split_index());
    #[allow(unreachable_code)]
    None
}

pub fn maybe_timer_segment_splitted(idx: i32) -> Option<bool> {
    #[cfg(feature = "unstable")]
    return Some(asr::timer::segment_splitted(idx));
    #[allow(unreachable_code)]
    None
}
