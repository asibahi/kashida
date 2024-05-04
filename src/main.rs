fn main() {
    static EXAMPLE_TEXT: &str = "والعصر إن الإنسان لفي خسر إلا الذين آمنوا وعملوا الصالحات";

    for bp in kashida::run(EXAMPLE_TEXT).iter() {
        println!("{bp}");
    }
}
