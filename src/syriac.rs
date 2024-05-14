#![allow(unused)]

use alloc::{boxed::Box, vec::Vec};
use core::iter;
use hashbrown::{hash_map::Entry, HashMap};
use itertools::Itertools;
use unicode_joining_type::{get_joining_group, JoiningGroup};

use crate::global::*;
use crate::KashidaCandidate;

fn is_alaph(c: char) -> bool {
    matches!(get_joining_group(c), JoiningGroup::Alaph)
}
fn is_lamadh(c: char) -> bool {
    matches!(get_joining_group(c), JoiningGroup::Lamadh)
}

// Useful resources: https://www.unicode.org/versions/Unicode15.0.0/ch09.pdf
//                   https://bug-attachments.documentfoundation.org/attachment.cgi?id=182206
#[must_use]
pub fn find_kashidas(input: &str) -> Box<[usize]> {
    let mut candidates: HashMap<_, KashidaCandidate> = HashMap::with_capacity(input.len() / 2);

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
            .pad_using(3, |_| None)
            .tuple_windows();

        for glyph_window in graphemes {
            find_kashidas_in_glyph_run(glyph_window, input, |kc| {
                match candidates.entry(word_idx) {
                    Entry::Occupied(mut e) if kc.bp_priority <= e.get().bp_priority => e.insert(kc),
                    Entry::Occupied(_) => kc,
                    Entry::Vacant(e) => *e.insert(kc),
                };
            });
        }
    }

    let mut ret = candidates.into_values().collect::<Vec<_>>();
    ret.sort_unstable_by_key(|a| a.bp_priority);
    ret.into_iter().map(|kc| kc.breakpoint).collect()
}

fn find_kashidas_in_glyph_run(
    (g1, g2, g3): (Option<&str>, Option<&str>, Option<&str>),
    input: &str,
    mut insert_candidate: impl FnMut(KashidaCandidate),
) {
    let breakpoint = |g: &str| g.as_ptr() as usize - input.as_ptr() as usize;
    match (g1, g2, g3) {
        // If Input contains Kashida, that's the place
        (_, Some(g), _) if g.chars().all(is_kashida) => {
            insert_candidate(KashidaCandidate::new(breakpoint(g) + g.len(), 0))
        }

        // deal with لا early
        (Some(lam), Some(alef), _) | (Some(_), Some(lam), Some(alef))
            if lam.contains(is_lamadh) && alef.contains(is_alaph) => {}

        (Some(preceding), Some(g), _)
            if preceding.contains(joins_following) && g.contains(joins_preceding) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(g), 1));
        }
        _ => {}
    }
}
