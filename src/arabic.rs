use alloc::{boxed::Box, vec::Vec};
use core::iter;
use hashbrown::{hash_map::Entry, HashMap};
use itertools::Itertools;
use unicode_joining_type::{get_joining_group, JoiningGroup};

use crate::global::*;
use crate::KashidaCandidate;

fn is_alef(c: char) -> bool {
    matches!(get_joining_group(c), JoiningGroup::Alef)
}
fn is_tooth(c: char) -> bool {
    matches!(
        get_joining_group(c),
        JoiningGroup::Beh
            | JoiningGroup::Noon
            | JoiningGroup::AfricanNoon
            | JoiningGroup::Nya
            | JoiningGroup::Yeh
            | JoiningGroup::FarsiYeh
            | JoiningGroup::ThinYeh
            | JoiningGroup::BurushaskiYehBarree
    )
}
fn is_dal(c: char) -> bool {
    matches!(get_joining_group(c), JoiningGroup::Dal)
}
fn is_reh(c: char) -> bool {
    matches!(get_joining_group(c), JoiningGroup::Reh)
}
fn is_seen_or_sad(c: char) -> bool {
    matches!(get_joining_group(c), JoiningGroup::Seen | JoiningGroup::Sad)
}
fn is_tah_or_kaf(c: char) -> bool {
    matches!(
        get_joining_group(c),
        JoiningGroup::Tah | JoiningGroup::Kaf | JoiningGroup::SwashKaf | JoiningGroup::Gaf
    )
}
fn is_ain_or_feh_or_qaf(c: char) -> bool {
    matches!(
        get_joining_group(c),
        JoiningGroup::Ain
            | JoiningGroup::Feh
            | JoiningGroup::AfricanFeh
            | JoiningGroup::Qaf
            | JoiningGroup::AfricanQaf
    )
}
fn is_lam(c: char) -> bool {
    matches!(get_joining_group(c), JoiningGroup::Lam)
}
fn is_meem(c: char) -> bool {
    matches!(get_joining_group(c), JoiningGroup::Meem)
}
fn is_noon(c: char) -> bool {
    matches!(
        get_joining_group(c),
        JoiningGroup::Noon | JoiningGroup::AfricanNoon | JoiningGroup::Nya
    )
}
fn is_heh(c: char) -> bool {
    matches!(
        get_joining_group(c),
        JoiningGroup::Heh | JoiningGroup::KnottedHeh | JoiningGroup::HehGoal
    )
}
fn is_teh_marbouta(c: char) -> bool {
    matches!(get_joining_group(c), JoiningGroup::TehMarbuta | JoiningGroup::HamzaOnHehGoal)
}
fn is_waw(c: char) -> bool {
    matches!(get_joining_group(c), JoiningGroup::Waw | JoiningGroup::StraightWaw)
}
fn is_final_yeh(c: char) -> bool {
    matches!(
        get_joining_group(c),
        JoiningGroup::Yeh
            | JoiningGroup::FarsiYeh
            | JoiningGroup::BurushaskiYehBarree
            | JoiningGroup::YehBarree
            | JoiningGroup::YehWithTail
            | JoiningGroup::RohingyaYeh
    )
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

// BIG MATCH based loosely on:
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
            if g1.contains(is_lam)
                && g2.contains(is_lam)
                && g3.contains(|c| is_heh(c) || is_teh_marbouta(c)) => {}

        // If Input contains Kashida, that's the place (unless the Kashida has a vowel on it)
        (_, Some(g), _, _) if g.chars().all(is_kashida) => {
            insert_candidate(KashidaCandidate::new(breakpoint(g), 0))
        }

        // deal with لا early
        // it is not counted as a grapheme for some reason
        (Some(lam), Some(alef), ..) if lam.contains(is_lam) && alef.contains(is_alef) => {}
        (Some(preceding), Some(lam), Some(alef), _)
        | (_, Some(preceding), Some(lam), Some(alef))
            if preceding.contains(joins_following)
                && lam.contains(is_lam)
                && alef.contains(is_alef) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(lam), 3));
        }
        (Some(_), Some(lam), Some(alef), _) if lam.contains(is_lam) && alef.contains(is_alef) => {}

        // heavy penalty on two letter words
        (Some(preceding), Some(g), None, None)
            if preceding.contains(joins_following) && g.contains(joins_preceding) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(g), 9));
        }

        // following ســـ or صـــ
        (Some(g1), Some(g2), ..) | (_, Some(g1), Some(g2), _)
            if g1.contains(is_seen_or_sad) && g2.contains(joins_preceding) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(g2), 1));
        }

        // before ـــبي or ـــيم
        (Some(preceding), Some(fst), Some(snd), None)
            if preceding.contains(joins_following)
                && fst.contains(is_tooth)
                && snd.contains(|c| is_final_yeh(c) || is_meem(c)) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(fst), 4));
        }
        // before ـــبن
        (Some(preceding), Some(fst), Some(snd), None)
            if preceding.contains(joins_following)
                && fst.contains(is_tooth)
                && snd.contains(is_noon) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(fst), 4));
        }

        // last letter in the word
        (_, Some(preceding), Some(g), None)
            if preceding.contains(joins_following) && g.contains(joins_preceding) =>
        // before ـــه
        {
            if g.contains(|c| is_heh(c) || is_teh_marbouta(c)) {
                insert_candidate(KashidaCandidate::new(breakpoint(g), 2));
            }
            // before ـــط or ـــل  or ـــك
            else if g.contains(|c| is_tah_or_kaf(c) || is_lam(c)) {
                insert_candidate(KashidaCandidate::new(breakpoint(g), 3));
            }
            // before ـع or ـق or ـف
            else if g.contains(is_ain_or_feh_or_qaf) {
                insert_candidate(KashidaCandidate::new(breakpoint(g), 5));
            }
            // before literally anything else
            else {
                insert_candidate(KashidaCandidate::new(breakpoint(g), 6));
            };
        }

        // if there is a connection between two letters.
        // before ــبر
        (Some(preceding), Some(fst), Some(snd), _)
            if preceding.contains(joins_following)
                && fst.contains(is_tooth)
                && snd.contains(is_reh) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(fst), 4));
        }

        // if there is a connection before one letter
        (Some(preceding), Some(g), ..) | (_, Some(preceding), Some(g), _)
            if preceding.contains(|c| joins_following(c) && !is_lam(c)) =>
        {
            let breakpoint = breakpoint(g);

            // Before a ــد or ــة
            if g.contains(|c| is_dal(c) || is_teh_marbouta(c) || is_heh(c)) {
                insert_candidate(KashidaCandidate::new(breakpoint, 2));
            }
            // before ــا
            else if g.contains(is_alef) {
                insert_candidate(KashidaCandidate::new(breakpoint, 4));
            }
            // before ــو
            else if g.contains(is_waw) {
                insert_candidate(KashidaCandidate::new(breakpoint, 5));
            }
            // before other things
            else {
                insert_candidate(KashidaCandidate::new(breakpoint, 7));
            }
        }

        _ => {} // don't add other things
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn jalala_no_candidates() {
        let input = "الله";
        let candidates = find_kashidas(input);

        assert_eq!(candidates.len(), 0);
    }

    #[test]
    fn basmala_candidates() {
        let input = "بسم الله الرحمن الرحيم";
        let candidates = find_kashidas(input);

        assert_eq!(candidates, vec![4, 37, 26].into_boxed_slice());
    }

    #[test]
    fn tawhid_candidates() {
        let input = "لا إله إلا الله";
        let candidates = find_kashidas(input);

        assert_eq!(candidates, vec![9].into_boxed_slice());
    }
}
