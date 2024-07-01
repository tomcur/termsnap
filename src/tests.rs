use clap::Parser;

use super::{run, Cli};

#[cfg(target_family = "unix")]
#[test]
fn echo() {
    let cli = Cli::parse_from([
        "termsnap",
        "-l",
        "20",
        "-c",
        "80",
        "--",
        "echo",
        "hello, world",
    ]);

    // create fake stdin and stdout that do nothing, otherwise the test is impacted by data on
    // stdin that is outside our control
    let (mut i, mut o) = std::os::unix::net::UnixStream::pair().unwrap();
    let screen = run(cli, &mut i, &mut o).unwrap();
    let content: String = screen.cells().map(|c| c.c).collect();

    assert_eq!(
        &content[..12],
        "hello, world",
        "terminal content was: {content:?}"
    );
}
