use alloc::{boxed::Box, vec::Vec};
use core::iter;
use hashbrown::{hash_map::Entry, HashMap};
use itertools::Itertools;
// use unicode_joining_type::{get_joining_group, JoiningGroup};

use crate::global::*;
use crate::KashidaCandidate;

#[must_use]
// this function is currently same code as in Arabic. WET because might change later?
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
            .chain(iter::once(None))
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
    (g1, g2, g3, g4): (Option<&str>, Option<&str>, Option<&str>, Option<&str>),
    input: &str,
    mut insert_candidate: impl FnMut(KashidaCandidate),
) {
    let breakpoint = |g: &str| g.as_ptr() as usize - input.as_ptr() as usize;
    match (g1, g2, g3, g4) {
        // heavy penalty on two letter words
        (Some(preceding), Some(g), None, None)
            if preceding.contains(joins_following) && g.contains(joins_preceding) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(g), 9));
        }

        // last letter in the word
        (_, Some(preceding), Some(g), None)
            if preceding.contains(joins_following) && g.contains(joins_preceding) =>
        {
            {
                insert_candidate(KashidaCandidate::new(breakpoint(g), 1));
            };
        }

        // any?
        (Some(preceding), Some(g), ..)
            if preceding.contains(joins_following) && g.contains(joins_preceding) =>
        {
            {
                insert_candidate(KashidaCandidate::new(breakpoint(g), 2));
            };
        }
        _ => {} // don't add other things
    }
}
