# Kashida

If you want to justify Arabic text, you eventually need to insert kashidas (Unicode character U+640, or ـ ) between letters. This mini-crate does a job at giving you hopefully decent looking candidates. Logic is based loosely on the discussion here: https://web.archive.org/web/20030719183154/http://www.microsoft.com/middleeast/msdn/JustifyingText-CSS.aspx

The library has currently one function, named `find_kashidas`. You give it an Arabic string, and it gives a sorted, by priority, list of Kashida location candidates, in byte index. Perfecty usable with `String::insert`. There is no verification done on whether the string is truly Arabic or not. It works for voweled texts fine.

Oh it is `no_std` as well.

Current plan before publishing on crates.io is to add some surrounding API to allow, potentially, extending the crate to other scripts (such as Syriac or Nko). The reason I am not adding those scripts myself is that I cannot read Syriac or Nko. But some better people might.
