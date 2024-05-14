use alloc::boxed::Box;
use core::iter;
use hashbrown::HashMap;
use itertools::Itertools;
use unicode_joining_type::{get_joining_type, JoiningType};

pub(crate) fn is_kashida(c: char) -> bool {
    matches!(get_joining_type(c), JoiningType::JoinCausing)
}
pub(crate) fn joins_following(c: char) -> bool {
    matches!(get_joining_type(c), JoiningType::DualJoining | JoiningType::JoinCausing)
}
pub(crate) fn joins_preceding(c: char) -> bool {
    matches!(
        get_joining_type(c),
        JoiningType::DualJoining | JoiningType::JoinCausing | JoiningType::RightJoining
    )
}

#[must_use]
pub fn find_kashidas(input: &str) -> Box<[usize]> {
    let mut candidates: HashMap<_, usize> = HashMap::with_capacity(input.len() / 2);

    let word_segmenter = icu_segmenter::WordSegmenter::new_auto();
    let grapheme_segmenter = icu_segmenter::GraphemeClusterSegmenter::new();

    let words = word_segmenter
        .segment_str(input)
        .tuple_windows()
        .filter_map(|(wb1, wb2)| Some(&input[wb1..wb2]).filter(|s| !s.trim().is_empty()));

    for (word_idx, word) in words.enumerate() {
        let graphemes = grapheme_segmenter
            .segment_str(word)
            .tuple_windows()
            .map(|(gb1, gb2)| Some(&word[gb1..gb2]))
            .pad_using(2, |_| None)
            .chain(iter::once(None))
            .tuple_windows();

        for glyph_window in graphemes {
            find_kashidas_in_glyph_run(glyph_window, input, |bp| {
                candidates.insert(word_idx, bp);
            });
        }
    }

    candidates.into_values().collect()
}

fn find_kashidas_in_glyph_run(
    (g1, g2, g3): (Option<&str>, Option<&str>, Option<&str>),
    input: &str,
    mut insert_candidate: impl FnMut(usize),
) {
    let breakpoint = |g: &str| g.as_ptr() as usize - input.as_ptr() as usize;
    match (g1, g2, g3) {
        (Some(preceding), Some(g), ..)
            if preceding.contains(joins_following) && g.contains(joins_preceding) =>
        {
            insert_candidate(breakpoint(g));
        }
        _ => {}
    }
}
