#![no_std]

extern crate alloc;

mod arabic;
mod ffi;
mod global;
mod syriac;

use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    vec::Vec,
};

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

/// Script to find Kashidas in. Only Arabic and Syriac for now.
/// Use Unknown to get the generic function.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum Script {
    Arabic,
    Syriac,
    Unknown,
}

/// Main entry point.
///
/// Does not verify string is valid for the language chosen.
///
/// Returns a list of byte-positions to insert the Kashida in, sorted by priority.
///
/// Does not guarantee a stable ordering for the same string. However, all positions are guaranteed to be valid.
/// If a Kashida is suggested at a wrong position, please report the bug.
#[must_use]
pub fn find_kashidas(input: &str, script: Script) -> Box<[usize]> {
    match script {
        Script::Arabic => arabic::find_kashidas(input),
        Script::Syriac => syriac::find_kashidas(input),
        Script::Unknown => global::find_kashidas(input),
    }
}

/// Convenience function to place the kashidas you found into your string.
///
/// To be used after `find_kashidas`. Make sure the same text is passed to both,
/// and the output of the first function is used. Doesn't allocate if it does not
/// have to.
///
/// Uses U+0640 ARABIC TATWEEL, which is used for most connected scripts.
pub fn place_kashidas<'a>(
    text: &'a str,
    kashida_locs: &'_ [usize],
    kashida_count: usize,
) -> Cow<'a, str> {
    if kashida_count == 0 || kashida_locs.is_empty() {
        Cow::Borrowed(text)
    } else {
        let mut buffer = text.to_owned();
        let mut locs = kashida_locs.iter().cycle().take(kashida_count).collect::<Vec<_>>();
        locs.sort_unstable_by(|a, b| b.cmp(a));
        for kc in locs {
            buffer.insert(*kc, 'ـ'); // e.g. N'Ko uses a different character (U+07FA: NKO LAJANYALAN)
        }
        Cow::Owned(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn basmala_placement() {
        let input = "بسم الله الرحمن الرحيم";
        let candidates = crate::arabic::find_kashidas(input);

        let output = place_kashidas(input, &candidates, 25);

        assert_eq!(candidates, vec![4, 37, 26].into_boxed_slice());
        assert_eq!(output, "بســـــــــم الله الرحمــــــــن الرحــــــــيم");
    }
}
