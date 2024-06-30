use clap::Parser;

use super::{run, Cli};

#[test]
fn bash_echo() {
    let cli = Cli::parse_from([
        "termsnap",
        "-l",
        "20",
        "-c",
        "80",
        "--",
        "bash",
        "-c",
        "echo 'hello, world'",
    ]);

    let screen = run(cli).unwrap();
    let content: String = screen.cells().map(|c| c.c).collect();

    assert_eq!(&content[..12], "hello, world");
}
