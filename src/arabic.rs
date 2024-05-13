use alloc::{boxed::Box, vec::Vec};
use constcat::concat_slices;
use core::iter;
use hashbrown::{hash_map::Entry, HashMap};
use itertools::Itertools;

use super::KashidaCandidate;

// Code points scoured from Unicode charts manually.
pub const KASHIDA: char = '\u{0640}';
pub const ALEFS: &[char] = &[
    '\u{0622}', '\u{0623}', '\u{0625}', '\u{0627}', '\u{0671}', '\u{0672}', '\u{0673}', '\u{0773}',
    '\u{0774}', '\u{0870}', '\u{0871}', '\u{0872}', '\u{0873}', '\u{0874}', '\u{0875}', '\u{0876}',
    '\u{0877}', '\u{0878}', '\u{0879}', '\u{087A}', '\u{087B}', '\u{087C}', '\u{087D}', '\u{087E}',
    '\u{087F}', '\u{0880}', '\u{0881}', '\u{0882}',
];
pub const BEHS: &[char] = &[
    '\u{0628}', '\u{062A}', '\u{062B}', '\u{066E}', '\u{0679}', '\u{067A}', '\u{067B}', '\u{067C}',
    '\u{067D}', '\u{067E}', '\u{067F}', '\u{0680}', '\u{0750}', '\u{0751}', '\u{0752}', '\u{0753}',
    '\u{0754}', '\u{0755}', '\u{0756}', '\u{08A0}', '\u{08A1}', '\u{08B6}', '\u{08B7}', '\u{08B8}',
    '\u{08BE}', '\u{08BF}', '\u{08C0}',
];
pub const JEEMS: &[char] = &[
    '\u{062C}', '\u{062D}', '\u{062E}', '\u{0681}', '\u{0682}', '\u{0683}', '\u{0684}', '\u{0685}',
    '\u{0686}', '\u{0687}', '\u{06BF}', '\u{0757}', '\u{0758}', '\u{076E}', '\u{076F}', '\u{0772}',
    '\u{077C}', '\u{08A2}', '\u{08C1}', '\u{08C5}', '\u{08C6}', '\u{088A}',
];
pub const DALS: &[char] = &[
    '\u{062F}', '\u{0630}', '\u{0688}', '\u{0689}', '\u{068A}', '\u{068B}', '\u{068C}', '\u{068D}',
    '\u{068E}', '\u{068F}', '\u{0690}', '\u{06EE}', '\u{0759}', '\u{075A}', '\u{08AE}',
];
pub const REHS: &[char] = &[
    '\u{0631}', '\u{0632}', '\u{0691}', '\u{0692}', '\u{0693}', '\u{0694}', '\u{0695}', '\u{0696}',
    '\u{0697}', '\u{0698}', '\u{0699}', '\u{06EF}', '\u{075B}', '\u{076B}', '\u{076C}', '\u{0771}',
    '\u{08AA}', '\u{08B2}', '\u{08B9}',
];
pub const SEENS: &[char] = &[
    '\u{0633}', '\u{0634}', '\u{069A}', '\u{069B}', '\u{069C}', '\u{06FA}', '\u{075C}', '\u{076D}',
    '\u{0770}', '\u{077D}', '\u{077E}',
];
pub const SADS: &[char] = &['\u{0635}', '\u{0636}', '\u{069D}', '\u{069E}', '\u{06FB}', '\u{08AF}'];
pub const TAHS: &[char] = &['\u{0637}', '\u{0638}', '\u{069F}', '\u{08A3}', '\u{088B}', '\u{088C}'];
pub const AINS: &[char] = &[
    '\u{0639}', '\u{063A}', '\u{06A0}', '\u{06FC}', '\u{075D}', '\u{075E}', '\u{075F}', '\u{08B3}',
    '\u{08C3}',
];
pub const FEHS: &[char] = &[
    '\u{0641}', '\u{06A1}', '\u{06A2}', '\u{06A3}', '\u{06A4}', '\u{06A5}', '\u{06A6}', '\u{0760}',
    '\u{0761}', '\u{08A4}', '\u{08BB}',
];
pub const QAFS: &[char] = &[
    '\u{0642}', '\u{066F}', '\u{06A7}', '\u{06A8}', '\u{08A5}', '\u{08B5}', '\u{08BC}', '\u{08C4}',
];
pub const KAFS: &[char] = &[
    '\u{063B}', '\u{063C}', '\u{0643}', '\u{06A9}', '\u{06AA}', '\u{06AB}', '\u{06AC}', '\u{06AD}',
    '\u{06AE}', '\u{06AF}', '\u{06B0}', '\u{06B1}', '\u{06B2}', '\u{06B3}', '\u{06B4}', '\u{0762}',
    '\u{0763}', '\u{0764}', '\u{077F}', '\u{08B0}', '\u{08B4}', '\u{08C2}', '\u{08C8}', '\u{088D}',
];
pub const LAMS: &[char] = &[
    '\u{0644}', '\u{06B5}', '\u{06B6}', '\u{06B7}', '\u{06B8}', '\u{076A}', '\u{08A6}', '\u{08C7}',
];
pub const MEEMS: &[char] = &['\u{0645}', '\u{0765}', '\u{0766}', '\u{08A7}'];
pub const NOONS: &[char] = &[
    '\u{0646}', '\u{06BA}', '\u{06BB}', '\u{06BC}', '\u{06BD}', '\u{0767}', '\u{0768}', '\u{0769}',
    '\u{08BD}', '\u{0889}',
];
pub const HEHS: &[char] = &['\u{0647}', '\u{06BE}', '\u{06C0}', '\u{06FF}'];
pub const TEH_MARBOUTA: &[char] = &['\u{0629}', '\u{06C1}', '\u{06C2}', '\u{06C3}'];
pub const WAWS: &[char] = &[
    '\u{0624}', '\u{0648}', '\u{06C4}', '\u{06C5}', '\u{06C6}', '\u{06C7}', '\u{06C8}', '\u{06C9}',
    '\u{06CA}', '\u{06CB}', '\u{06CF}', '\u{0778}', '\u{0779}', '\u{08AB}', '\u{08B1}',
];
pub const YEHS: &[char] = &[
    '\u{0626}', '\u{0620}', '\u{063D}', '\u{063E}', '\u{063F}', '\u{0649}', '\u{064A}', '\u{06CC}',
    '\u{06CD}', '\u{06CE}', '\u{06D0}', '\u{06D1}', '\u{06D2}', '\u{06D3}', '\u{0775}', '\u{0776}',
    '\u{0777}', '\u{077A}', '\u{077B}', '\u{08A8}', '\u{08A9}', '\u{08AC}', '\u{08BA}',
];
// pub const NON_CONNECTORS: &[char] = concat_slices!([char]: ALEFS, REHS, DALS, TEH_MARBOUTA, WAWS);
pub const TEETH: &[char] = concat_slices!([char]: BEHS, NOONS, YEHS);
pub const CONNECTORS_EXCEPT_LAMS: &[char] = concat_slices!([char]: TEETH, JEEMS, SEENS, SADS, TAHS, AINS, FEHS, QAFS, KAFS, MEEMS, NOONS, HEHS,);
pub const ALL_CONNECTORS: &[char] = concat_slices!([char]: CONNECTORS_EXCEPT_LAMS, LAMS);

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
            if g1.contains(|c| LAMS.contains(&c))
                && g2.contains(|c| LAMS.contains(&c))
                && g3.contains(|c| HEHS.contains(&c)) => {}

        // If Input contains Kashida, that's the place (unless the Kashida has a vowel on it)
        (_, Some(g), _, _) if g.chars().all(|c| c == KASHIDA) => {
            insert_candidate(KashidaCandidate::new(breakpoint(g), 0));
        }

        // heavy penalty on two letter words
        (Some(preceding), Some(g), None, None)
            if preceding.contains(|c| CONNECTORS_EXCEPT_LAMS.contains(&c))
                || (preceding.contains(|c| LAMS.contains(&c))
                    && g.chars().all(|c| !ALEFS.contains(&c))) =>
        {
            insert_candidate(KashidaCandidate::new(breakpoint(g), 9));
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

        // if there is a connection between two letters.
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
                insert_candidate(KashidaCandidate::new(breakpoint, 4));
            }
            // before ــو
            else if g.contains(|c| WAWS.contains(&c)) {
                insert_candidate(KashidaCandidate::new(breakpoint, 5));
            }
            // before other things
            else if preceding.chars().all(|c| !LAMS.contains(&c)) {
                insert_candidate(KashidaCandidate::new(breakpoint, 7));
            }
        }

        _ => {} // don't add other things
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::{println, vec};

    use super::*;

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

        println!("{:?}", candidates);
        assert_eq!(candidates, vec![4, 37, 26].into_boxed_slice());
    }
}
