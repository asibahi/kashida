use kashida::place_kashidas;

fn main() {
    static EXAMPLE_TEXT: &str = "والعصر إن الإنسان لفي خسر إلا الذين آمنوا وعملوا الصالحات";

    let candidates = kashida::find_kashidas(EXAMPLE_TEXT, kashida::Script::Arabic);
    println!("{:?}", candidates);

    let count = 6;
    let input = place_kashidas(EXAMPLE_TEXT, &candidates, count);

    println!("{input}");
}
