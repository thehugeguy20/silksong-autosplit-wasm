use alloc::vec::Vec;

pub fn maybe_timer_current_attempt_segments_splitted() -> Option<Vec<bool>> {
    #[cfg(feature = "unstable")]
    return Some(asr::timer::current_attempt_segments_splitted());
    #[allow(unreachable_code)]
    None
}
