use std::{
    collections::HashMap,
    io::{Read, Write},
    os::fd::AsFd,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

use alacritty_terminal::{
    event::OnResize,
    tty::{EventedPty, EventedReadWrite, Pty},
};
use clap::Parser;
use rustix::termios;

use termsnap_lib::{Screen, Term};

mod poll;
mod ringbuffer;
use ringbuffer::{IoResult, Ringbuffer};

const DEFAULT_NUM_LINES: u16 = 24;
const DEFAULT_NUM_COLUMNS: u16 = 80;

/// Set the file descriptor to non-blocking.
fn set_nonblocking(fd: impl AsFd) -> anyhow::Result<()> {
    let mut flags = rustix::fs::fcntl_getfl(fd.as_fd())?;
    flags |= rustix::fs::OFlags::NONBLOCK;
    rustix::fs::fcntl_setfl(fd.as_fd(), flags)?;

    Ok(())
}

/// Execute the callback with the attributes of the terminal corresponding to the file descriptor
/// set to raw. When the callback finishes the terminal attributes are reset.
fn with_raw<F: AsFd, R>(mut fd: F, f: impl FnOnce(&mut F) -> R) -> R {
    let orig_attrs = termios::tcgetattr(fd.as_fd()).expect("could not get terminal attributes");
    let mut attrs = orig_attrs.clone();
    attrs.make_raw();
    termios::tcsetattr(fd.as_fd(), termios::OptionalActions::Now, &attrs)
        .expect("could not set terminal attributes");
    let r = f(&mut fd);
    termios::tcsetattr(fd.as_fd(), termios::OptionalActions::Now, &orig_attrs)
        .expect("could not set terminal attributes");
    r
}

/// Create an SVG of a command's output by running it in a pseudo-terminal (PTY) and interpreting
/// the command's output by an in-memory terminal emulator.
#[derive(Debug, clap::Parser)]
#[command(version)]
struct Cli {
    /// Run the command interactively. This prevents the SVG from being output on standard output.
    /// Use `--out` to specify a file for storing the SVG.
    ///
    /// This connects the command's pseudo-terminal (PTY) to the standard input and output of the
    /// termsnap process. Note: this does not perform ANSI escape sequence translation.
    ///
    /// This can also be used for piping output (non-interactively) to and from the command.
    #[arg(short, long)]
    interactive: bool,

    /// A location for storing the resulting SVG.
    #[arg(short, long)]
    out: Option<PathBuf>,

    /// The command to run. Its output will be turned into an SVG.
    command: String,

    /// The number of lines in the emulated terminal. If unset, this defaults to value of the LINES
    /// environment variable if set, or 24 otherwise.
    ///
    /// This setting is ignored if `--interactive` is set.
    #[arg(short, long)]
    lines: Option<u16>,

    /// The number of columns in the emulated terminal. If unset, this defaults to value of the
    /// COLUMNS enviornment variable if set, or 80 otherwise.
    ///
    /// This setting is ignored if `--interactive` is set.
    #[arg(short, long)]
    columns: Option<u16>,

    /// Arguments provided to the command.
    #[arg(trailing_var_arg(true))]
    args: Option<Vec<String>>,
}

/// run the command in the pty non-interactively: i.e., simply read its stdout
fn read_pty(pty: &mut Pty, term: &mut Term) -> Screen {
    let reader = pty.reader();

    for byte in reader.bytes() {
        match byte {
            Ok(byte) => term.process(byte),
            Err(err) => {
                if !matches!(
                    err.kind(),
                    std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock
                ) {
                    break;
                }
            }
        }
    }

    term.current_screen()
}

/// run the command in the pty interactively by proxying between its and termsnap's stdin and
/// stdout
fn proxy_pty(pty: &mut Pty, term: &mut Term) -> anyhow::Result<Screen> {
    let window_size_changed = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(
        signal_hook::consts::signal::SIGWINCH,
        window_size_changed.clone(),
    )
    .expect("failed to set signal handler");

    let parent_stdin = std::io::stdin();
    let parent_stdout = std::io::stdout();

    let screen = with_raw(parent_stdout, move |parent_stdout| {
        let mut parent_stdin = parent_stdin.lock();
        let mut parent_stdout = parent_stdout.lock();

        // buffers between parent and pty's stdin/stdout pairs
        let mut stdin_buf = Ringbuffer::<4096>::new();
        let mut stdout_buf = Ringbuffer::<4096>::new();

        loop {
            if let Some(alacritty_terminal::tty::ChildEvent::Exited(_code)) = pty.next_child_event()
            {
                break;
            }

            if window_size_changed.load(std::sync::atomic::Ordering::Relaxed) {
                window_size_changed.store(false, std::sync::atomic::Ordering::Relaxed);

                let winsize = termios::tcgetwinsize(std::io::stdout())?;
                let lines = winsize.ws_row;
                let columns = winsize.ws_col;

                pty.on_resize(alacritty_terminal::event::WindowSize {
                    num_lines: lines,
                    num_cols: columns,
                    cell_width: 1,
                    cell_height: 1,
                });
                term.resize(lines, columns);
            }

            let poll_result = match poll::poll([
                (!stdin_buf.is_full())
                    .then(|| PollFd::from_borrowed_fd(parent_stdin.as_fd(), PollFlags::IN)),
                (!stdout_buf.is_full())
                    .then(|| PollFd::from_borrowed_fd(pty.file().as_fd(), PollFlags::IN)),
                (!stdin_buf.is_empty())
                    .then(|| PollFd::from_borrowed_fd(pty.file().as_fd(), PollFlags::OUT)),
                (!stdout_buf.is_empty())
                    .then(|| PollFd::from_borrowed_fd(parent_stdout.as_fd(), PollFlags::OUT)),
            ]) {
                Ok(r) => r,
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::Interrupted {
                        continue;
                    } else {
                        anyhow::bail!(err);
                    }
                }
            };

            if poll_result[0] {
                let _ = stdin_buf.read(&mut parent_stdin);
            }

            if poll_result[1] {
                let pty_stdout = pty.reader();
                let res = stdout_buf.read(pty_stdout);
                for byte in res.bytes() {
                    term.process(byte);
                }
            }

            if poll_result[2] {
                let pty_stdin = pty.writer();
                let _ = stdin_buf.write(pty_stdin);
            }

            if poll_result[3] {
                match stdout_buf.write(&mut parent_stdout) {
                    IoResult::Ok(bytes) | IoResult::EOF(bytes) => {
                        if bytes.len() > 0 {
                            parent_stdout.flush().unwrap();
                        }
                    }
                    IoResult::Err { .. } => {}
                }
            }
        }

        anyhow::Ok(term.current_screen())
    })?;

    Ok(screen)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.interactive {
        if cli.out.is_none() {
            anyhow::bail!("`--interactive` is set but no SVG output file is specified in `--out`. See `termsnap --help`.");
        }

        if cli.lines.is_some() || cli.columns.is_some() {
            eprintln!("Warning: Setting `--lines` and `--columns` has no effect when `--interactive` is set");
        }

        if !std::io::stdin().is_terminal() {
            eprintln!("Warning: `--interactive` is set, but stdin is not a tty")
        }

        if !std::io::stdout().is_terminal() {
            eprintln!("Warning: `--interactive` is set, but stdout is not a tty")
        }
    }

    let (lines, columns) = if cli.interactive {
        termios::tcgetwinsize(std::io::stdout())
            .map(|winsize| (winsize.ws_row, winsize.ws_col))
            .unwrap_or((DEFAULT_NUM_LINES, DEFAULT_NUM_COLUMNS))
    } else {
        let lines: u16 = cli
            .lines
            .or_else(|| {
                std::env::var("LINES")
                    .ok()
                    .and_then(|lines| lines.parse().ok())
            })
            .unwrap_or(DEFAULT_NUM_LINES);
        let columns: u16 = cli
            .columns
            .or_else(|| {
                std::env::var("COLUMNS")
                    .ok()
                    .and_then(|columns| columns.parse().ok())
            })
            .unwrap_or(DEFAULT_NUM_COLUMNS);
        (lines, columns)
    };

    let mut pty = alacritty_terminal::tty::new(
        &alacritty_terminal::tty::Options {
            shell: Some(alacritty_terminal::tty::Shell::new(
                cli.command,
                cli.args.unwrap_or(vec![]),
            )),
            working_directory: None,
            hold: false,
            env: {
                let mut env = HashMap::new();
                env.insert("LINES".to_owned(), lines.to_string());
                env.insert("COLUMNS".to_owned(), columns.to_string());
                env.insert("TERM".to_owned(), "linux".to_owned());
                env
            },
        },
        alacritty_terminal::event::WindowSize {
            num_lines: lines.into(),
            num_cols: columns.into(),
            cell_width: 1,
            cell_height: 1,
        },
        0,
    )
    .unwrap();

    let mut term = Term::new(lines, columns);

    let screen = if cli.interactive {
        proxy_pty(&mut pty, &mut term)?
    } else {
        read_pty(&mut pty, &mut term)
    };

    if let Some(out) = cli.out {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(out)?;
        write!(file, "{}", screen.to_svg())?;
    } else {
        println!("{}", screen.to_svg())
    }

    Ok(())
}
