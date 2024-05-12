fn main() {
    static EXAMPLE_TEXT: &str = "والعصر إن الإنسان لفي خسر إلا الذين آمنوا وعملوا الصالحات";

    let candidates = kashida::find_kashidas(EXAMPLE_TEXT, kashida::Script::Arabic);
    println!("{:?}", candidates);

    // How to insert
    let count = 6;

    let mut input = EXAMPLE_TEXT.to_owned();
    let mut candidates = candidates.iter().take(count).collect::<Vec<_>>();
    candidates.sort_by(|a, b| b.cmp(a));

    for c in candidates {
        input.insert(*c, 'ـ');
    }

    println!("{input}");
}
