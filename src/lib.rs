#![no_std]

extern crate alloc;

mod codepoints;

use alloc::{boxed::Box, vec::Vec};
use codepoints::*;
use core::iter;
use hashbrown::{hash_map::Entry, HashMap};
use itertools::Itertools;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
struct KashidaCandidate {
    /// where the candidate is
    breakpoint: usize,

    /// lower is better
    bp_priority: usize,
}

impl KashidaCandidate {
    fn new(breakpoint: usize, bp_priority: usize) -> Self {
        Self { breakpoint, bp_priority }
    }
}

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
            .chain(iter::once(None))
            .tuple_windows();

        for glyph_window in graphemes {
            find_kashidas_in_glyph_run(glyph_window, input, |kc| {
                match candidates.entry(word_idx) {
                    Entry::Occupied(mut e) if kc.bp_priority <= e.get().bp_priority => e.insert(kc),
                    Entry::Vacant(e) => *e.insert(kc),
                    _ => kc,
                };
            });
        }
    }

    let mut ret = candidates.into_values().collect::<Vec<_>>();
    ret.sort_unstable_by_key(|a| a.bp_priority);
    ret.into_iter().map(|kc| kc.breakpoint).collect()
}

// BIG MATCH based on:
// https://web.archive.org/web/20030719183154/http://www.microsoft.com/middleeast/msdn/JustifyingText-CSS.aspx
fn find_kashidas_in_glyph_run(
    (g1, g2, g3, g4): (Option<&str>, Option<&str>, Option<&str>, Option<&str>),
    input: &str,
    mut insert_candidate: impl FnMut(KashidaCandidate),
) {
    let breakpoint = |g: &str| g.as_ptr() as usize - input.as_ptr() as usize;
    match (g1, g2, g3, g4) {
        // skip لفظ الجلالة
        (Some(g1), Some(g2), Some(g3), None)
            if g1.contains(|c| LAMS.contains(&c))
                && g2.contains(|c| LAMS.contains(&c))
                && g3.contains(|c| HEHS.contains(&c)) => {}

        // If Input contains Kashida, that's the place
        (_, Some(g), _, _) if g.contains(KASHIDA) => {
            insert_candidate(KashidaCandidate::new(breakpoint(g), 0));
        }

        // following ســـ or صـــ
        (Some(g1), Some(g2), ..) | (_, Some(g1), Some(g2), _)
            if g1.contains(|c| SADS.contains(&c) || SEENS.contains(&c))
                && g2.contains(char::is_alphabetic) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(g2), 1));
        }

        // before ـــبي or ـــيم
        (Some(preceding), Some(fst), Some(snd), None)
            if preceding.contains(|c| ALL_CONNECTORS.contains(&c))
                && fst.contains(|c| TEETH.contains(&c))
                && snd.contains(|c| YEHS.contains(&c) || MEEMS.contains(&c)) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(fst), 4));
        }

        // last letter in the word
        (_, Some(preceding), Some(g), None)
            if preceding.contains(|c| CONNECTORS_EXCEPT_LAMS.contains(&c)) =>
        // before ـــه
        {
            if g.contains(|c| HEHS.contains(&c)) {
                insert_candidate(KashidaCandidate::new(breakpoint(g), 2));
            }
            // before ـــط or ـــل  or ـــك
            else if g.contains(|c| TAHS.contains(&c) || LAMS.contains(&c) || KAFS.contains(&c)) {
                insert_candidate(KashidaCandidate::new(breakpoint(g), 3));
            }
            // before ـع or ـق or ـف
            else if g.contains(|c| AINS.contains(&c) || FEHS.contains(&c) || QAFS.contains(&c)) {
                insert_candidate(KashidaCandidate::new(breakpoint(g), 5));
            }
            // before literally anything else
            else {
                insert_candidate(KashidaCandidate::new(breakpoint(g), 6));
            };
        }

        // if there is a connection between two lettets.
        // before ــلا  . It is *not* counted as an indivisible grapheme for some reason.
        (Some(preceding), Some(fst), Some(snd), _)
            if preceding.contains(|c| ALL_CONNECTORS.contains(&c))
                && fst.contains(|c| LAMS.contains(&c))
                && snd.contains(|c| ALEFS.contains(&c)) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(fst), 3));
        }
        // before ــبر
        (Some(preceding), Some(fst), Some(snd), _)
            if preceding.contains(|c| ALL_CONNECTORS.contains(&c))
                && fst.contains(|c| TEETH.contains(&c))
                && snd.contains(|c| REHS.contains(&c)) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(fst), 4));
        }

        // if there is a connection before one letter
        (Some(preceding), Some(g), ..) | (_, Some(preceding), Some(g), _)
            if preceding.contains(|c| ALL_CONNECTORS.contains(&c)) =>
        {
            let breakpoint = breakpoint(g);

            // Before a ــد or ــة
            if g.contains(|c| DALS.contains(&c) || TEH_MARBOUTA.contains(&c)) {
                insert_candidate(KashidaCandidate::new(breakpoint, 2));
            }
            // before ــا but not within لا
            else if preceding.chars().all(|c| !LAMS.contains(&c))
                && g.contains(|c| ALEFS.contains(&c))
            {
                insert_candidate(KashidaCandidate::new(breakpoint, 1));
            }
            // before ــو
            else if g.contains(|c| WAWS.contains(&c)) {
                insert_candidate(KashidaCandidate::new(breakpoint, 5));
            }
        }

        _ => {} // don't add other things
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    #[test]
    fn test1() {
        todo!()
    }
}
