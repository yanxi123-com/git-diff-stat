fn main() {
    let show_help = std::env::args().nth(1).as_deref() == Some("--help");

    if show_help {
        print!("{}", git_diff_stat::HELP_TEXT);
        return;
    }

    eprintln!("not implemented");
    std::process::exit(1);
}
