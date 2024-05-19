
![Crates.io Version](https://img.shields.io/crates/v/kashida)
![docs.rs](https://img.shields.io/docsrs/kashida)

# Kashida

If you want to justify Arabic (or Syriac, or any other connected script) text, you eventually need to insert kashidas (Unicode character U+640, or ـ ) between letters. This mini-crate does a job at giving you hopefully decent looking candidates. Logic for Arabic is based loosely on the [Microsoft discussion here](https://web.archive.org/web/20030719183154/http://www.microsoft.com/middleeast/msdn/JustifyingText-CSS.aspx). Syriac is based on [this document](https://bug-attachments.documentfoundation.org/attachment.cgi?id=182206).

The main entry point of the library is a `find_kashidas`. You give it a string, and it gives a sorted, by priority, list of Kashida location candidates, in byte index. Perfecty usable with `String::insert`, or the convenience function provided of `place_kashidas`. There is no verification done on whether the string is truly the script you say it is or not. It works for voweled texts fine.

Oh it is `no_std` as well.

The `Script` enum has `Arabic`, `Syriac`, and `Unknown`. Arabic and Syriac have custom rules and priorities, but if you use the `Unknown` variant you'd get a generic set of rules that  should, in theory, work for other scripts. If you can read and contribute these other scripts, help would be most welcome.

I tried to add a couple of C FFI functions, with help from Rust Discord. However, I don't understand C enough to know how to use them. If you can try them and let me know how to improve them, it would be very helpful.
