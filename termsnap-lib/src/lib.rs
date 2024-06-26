use std::{
    fmt::{Display, Write},
    sync::OnceLock,
};

use alacritty_terminal::{
    term::{
        cell::{Cell, Flags},
        color::Colors,
        test::TermSize,
        Config, Term as AlacrittyTerm,
    },
    vte::{
        self,
        ansi::{Color, Processor},
    },
};

mod colors;

// ideally users can define their own colors
static COLORS: OnceLock<Colors> = OnceLock::new();

const FONT_ASPECT_RATIO: f32 = 0.6;
const FONT_ASCENT: f32 = 0.750;

#[derive(PartialEq)]
struct Style {
    fg: Color,
    bg: Color,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

impl Style {
    /// private conversion from alacritty Cell to Style
    fn from_cell(cell: &Cell) -> Self {
        Style {
            fg: cell.fg,
            bg: cell.bg,

            bold: cell.flags.intersects(Flags::BOLD),
            italic: cell.flags.intersects(Flags::ITALIC),
            underline: cell.flags.intersects(Flags::ALL_UNDERLINES),
            strikethrough: cell.flags.intersects(Flags::STRIKEOUT),
        }
    }
}

struct ColorDisplayWrapper(Color);

impl Display for ColorDisplayWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Color::Named(named_color) => {
                let colors = COLORS.get_or_init(|| colors::colors());
                f.write_fmt(format_args!("{}", colors[named_color as usize].unwrap()))
            }
            Color::Spec(rgb) => {
                write!(f, "{}", rgb)
            }
            Color::Indexed(idx) => {
                let colors = COLORS.get_or_init(|| colors::colors());
                f.write_fmt(format_args!("{}", colors[idx as usize].unwrap()))
            }
        }
    }
}

struct TextLine {
    text: Vec<char>,
}

impl TextLine {
    fn with_capacity(capacity: usize) -> Self {
        TextLine {
            text: Vec::with_capacity(capacity),
        }
    }

    fn push_cell(&mut self, char: char) {
        self.text.push(char);
    }

    fn clear(&mut self) {
        self.text.clear();
    }

    fn len(&self) -> usize {
        self.text.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the character cells of this text line, discarding trailing whitespace.
    fn chars(&self) -> &[char] {
        let trailing_whitespace_chars = self
            .text
            .iter()
            .rev()
            .position(|c| !c.is_whitespace())
            .unwrap_or(self.text.len());
        let end = self.text.len() - trailing_whitespace_chars;
        &self.text[..end]
    }
}

fn fmt_rect(
    f: &mut std::fmt::Formatter<'_>,
    x0: u16,
    y0: u16,
    x1: u16,
    y1: u16,
    color: Color,
) -> std::fmt::Result {
    write!(
        f,
        r#"<rect x="{x}" y="{y}" width="{width}" height="{height}" style="fill: {color};" />"#,
        x = f32::from(x0) * FONT_ASPECT_RATIO,
        y = y0,
        width = f32::from(x1 - x0 + 1) * FONT_ASPECT_RATIO,
        height = y1 - y0 + 1,
        color = ColorDisplayWrapper(color),
    )
}

fn fmt_text(
    f: &mut std::fmt::Formatter<'_>,
    x: u16,
    y: u16,
    text: &TextLine,
    style: &Style,
) -> std::fmt::Result {
    let chars = text.chars();
    let text_length = chars.len() as f32 * FONT_ASPECT_RATIO;
    write!(
        f,
        r#"<text x="{x}" y="{y}" textLength="{text_length}" style="fill: {color};"#,
        x = f32::from(x) * FONT_ASPECT_RATIO,
        y = f32::from(y) + FONT_ASCENT,
        color = ColorDisplayWrapper(style.fg),
    )?;

    if style.bold {
        f.write_str(" font-weight: 600;")?;
    }
    if style.italic {
        f.write_str(" font-style: italic;")?;
    }
    if style.underline || style.strikethrough {
        f.write_char(' ')?;
        if style.underline {
            f.write_str(" underline")?;
        }
        if style.strikethrough {
            f.write_str(" line-through")?;
        }
    }

    f.write_str(r#"">"#)?;
    for char in chars {
        match *char {
            // non-breaking space
            ' ' => f.write_str("&#160;")?,
            // escape tag opening
            '<' => f.write_str("&#x3C;")?,
            c => f.write_char(c)?,
        }
    }
    f.write_str("</text>")?;
    Ok(())
}

/// A static snapshot of a terminal screen.
pub struct Screen {
    lines: u16,
    cols: u16,
    cells: Vec<Cell>,
}

impl Screen {
    /// Get a [std::fmt::Display] that prints an SVG when formatted. Set `fonts` to specify fonts
    /// to be included in the SVG's `font-family` style. `font-family` always includes `monospace`.
    ///
    /// The SVG is generated once [std::fmt::Display::fmt] is called.
    pub fn to_svg<'s, 'f>(&'s self, fonts: &'f [&'f str]) -> impl Display + 's
    where
        'f: 's,
    {
        struct Svg<'s> {
            screen: &'s Screen,
            fonts: &'s [&'s str],
        }

        impl<'s> Display for Svg<'s> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let Screen {
                    lines,
                    cols,
                    ref cells,
                } = self.screen;

                write!(
                    f,
                    r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
                    f32::from(self.screen.cols) * FONT_ASPECT_RATIO,
                    lines,
                )?;

                f.write_str(
                    "
<style>
  .screen {
    font-family: ",
                )?;

                for font in self.fonts {
                    f.write_char('"')?;
                    f.write_str(font)?;
                    f.write_str("\", ")?;
                }

                f.write_str(
                    r#"monospace;
    font-size: 1px;
  }
</style>
<g class="screen">
"#,
                )?;

                // find background rectangles to draw by greedily flooding lines then flooding down columns
                let mut drawn = vec![false; usize::from(*lines) * usize::from(*cols)];
                for y0 in 0..*lines {
                    for x0 in 0..*cols {
                        let idx = self.screen.idx(y0, x0);

                        if drawn[idx] {
                            continue;
                        }

                        let cell = &cells[idx];
                        let bg = cell.bg;
                        let mut end_x = x0;
                        let mut end_y = y0;

                        for x1 in x0 + 1..*cols {
                            let idx = self.screen.idx(y0, x1);
                            let cell = &cells[idx];
                            if cell.bg == bg {
                                end_x = x1;
                            } else {
                                break;
                            }
                        }

                        for y1 in y0 + 1..*lines {
                            let mut all = true;
                            for x1 in x0 + 1..*cols {
                                let idx = self.screen.idx(y1, x1);
                                let cell = &cells[idx];
                                if cell.bg != bg {
                                    all = false;
                                    break;
                                }
                            }
                            if !all {
                                break;
                            }
                            end_y = y1;
                        }

                        {
                            for y in y0..=end_y {
                                for x in x0..=end_x {
                                    let idx = self.screen.idx(y, x);
                                    drawn[idx] = true;
                                }
                            }
                        }

                        fmt_rect(f, x0, y0, end_x, end_y, bg)?;
                    }
                }

                // write text
                let mut text_line = TextLine::with_capacity(usize::from(*cols).next_power_of_two());
                for y in 0..*lines {
                    let idx = self.screen.idx(y, 0);
                    let cell = &cells[idx];
                    let mut style = Style::from_cell(cell);
                    let mut start_x = 0;

                    for x in 0..*cols {
                        let idx = self.screen.idx(y, x);
                        let cell = &cells[idx];
                        let style_ = Style::from_cell(cell);

                        if style_ != style {
                            if !text_line.is_empty() {
                                fmt_text(f, start_x, y, &text_line, &style)?;
                            }
                            text_line.clear();
                            style = style_;
                        }

                        if text_line.is_empty() {
                            start_x = x;
                            if cell.c == ' ' {
                                continue;
                            }
                        }

                        text_line.push_cell(cell.c);
                    }

                    if !text_line.is_empty() {
                        fmt_text(f, start_x, y, &text_line, &style)?;
                        text_line.clear();
                    }
                }

                f.write_str(
                    "
</g>
</svg>",
                )?;

                Ok(())
            }
        }

        Svg {
            screen: self,
            fonts,
        }
    }

    #[inline(always)]
    fn idx(&self, y: u16, x: u16) -> usize {
        usize::from(y) * usize::from(self.cols) + usize::from(x)
    }

    /// The number of screen lines in this snapshot.
    pub fn lines(&self) -> u16 {
        self.lines
    }

    /// The number of screen columns in this snapshot.
    pub fn cols(&self) -> u16 {
        self.cols
    }
}

pub trait PtyWriter {
    fn write(&mut self, text: String);
}

impl<F: FnMut(String)> PtyWriter for F {
    fn write(&mut self, text: String) {
        self(text)
    }
}

pub struct VoidPtyWriter;

impl PtyWriter for VoidPtyWriter {
    fn write(&mut self, _text: String) {}
}

struct EventProxy<Ev> {
    handler: std::cell::RefCell<Ev>,
}

impl<W: PtyWriter> alacritty_terminal::event::EventListener for EventProxy<W> {
    fn send_event(&self, event: alacritty_terminal::event::Event) {
        use alacritty_terminal::event::Event as AEvent;
        match event {
            AEvent::PtyWrite(text) => self.handler.borrow_mut().write(text),
            _ev => {}
        }
    }
}

/// An in-memory terminal emulator.
pub struct Term<Ev: PtyWriter> {
    lines: u16,
    columns: u16,
    term: AlacrittyTerm<EventProxy<Ev>>,
    processor: vte::ansi::Processor<vte::ansi::StdSyncHandler>,
}

impl<W: PtyWriter> Term<W> {
    /// Create a new emulated terminal with a cell matrix of `lines` by `columns`.
    ///
    /// `pty_writer` is used to send output from the emulated terminal in reponse to ANSI requests.
    /// Use `[VoidWriter]` if you do not need to send responses to status requests.
    pub fn new(lines: u16, columns: u16, pty_writer: W) -> Self {
        let term = AlacrittyTerm::new(
            Config::default(),
            &TermSize {
                columns: columns.into(),
                screen_lines: lines.into(),
            },
            EventProxy {
                handler: pty_writer.into(),
            },
        );

        Term {
            lines,
            columns,
            term,
            processor: Processor::new(),
        }
    }

    /// Process one byte of ANSI-escaped terminal data.
    pub fn process(&mut self, byte: u8) {
        self.processor.advance(&mut self.term, byte);
    }

    /// Resize the terminal screen to the specified dimension.
    pub fn resize(&mut self, lines: u16, columns: u16) {
        let new_size = TermSize {
            columns: columns.into(),
            screen_lines: lines.into(),
        };
        self.term.resize(new_size);
    }

    /// Get a snapshot of the current terminal screen.
    pub fn current_screen(&self) -> Screen {
        Screen {
            lines: self.lines,
            cols: self.columns,
            cells: self
                .term
                .grid()
                .display_iter()
                .map(|point_cell| point_cell.cell.clone())
                .collect(),
        }
    }
}

/// Feed an ANSI sequence through a terminal emulator, returning the resulting terminal screen contents.
pub fn emulate(lines: u16, columns: u16, ansi_sequence: &[u8]) -> Screen {
    let mut term = Term::new(lines, columns, VoidPtyWriter);
    for &byte in ansi_sequence {
        term.process(byte);
    }
    term.current_screen()
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let screen = super::emulate(24, 80, include_bytes!("./tests/ls.txt"));
        let expected = "total 60
drwxr-xr-x  6 thomas users  4096 Jun 19 15:58 .
drwxr-xr-x 34 thomas users  4096 Jun 16 10:28 ..
-rw-r--r--  1 thomas users 19422 Jun 18 17:22 Cargo.lock
-rw-r--r--  1 thomas users   749 Jun 19 11:33 Cargo.toml
-rw-r--r--  1 thomas users  1940 Jun 16 11:19 flake.lock
-rw-r--r--  1 thomas users   640 Jun 16 11:19 flake.nix
drwxr-xr-x  7 thomas users  4096 Jun 16 11:19 .git
-rw-r--r--  1 thomas users   231 Jun 16 11:30 README.md
drwxr-xr-x  2 thomas users  4096 Jun 19 12:20 src
drwxr-xr-x  3 thomas users  4096 Jun 18 14:36 target
drwxr-xr-x  3 thomas users  4096 Jun 18 11:22 termsnap-lib";

        let mut line = 0;
        let mut column = 0;

        for c in expected.chars() {
            match c {
                '\n' => {
                    for column in column..80 {
                        let idx = screen.idx(line, column);
                        assert_eq!(screen.cells[idx].c, ' ', "failed at {line}x{column}");
                    }
                    column = 0;
                    line += 1;
                }
                _ => {
                    let idx = screen.idx(line, column);
                    assert_eq!(screen.cells[idx].c, c, "failed at {line}x{column}");
                    column += 1;
                }
            }
        }
    }
}
