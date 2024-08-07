//! In-memory emulation of ANSI-escaped terminal data and rendering emulated terminal screens to
//! SVG files.
//!
//! ```rust
//! use termsnap_lib::{FontMetrics, Term, VoidPtyWriter};
//!
//! // Create a new terminal emulator and process some bytes.
//! let mut term = Term::new(24, 80, VoidPtyWriter);
//! for byte in b"a line of \x1B[32mcolored\x1B[0m terminal data" {
//!     term.process(*byte);
//! }
//!
//! // Create a snapshot of the terminal screen grid.
//! let screen = term.current_screen();
//!
//! let text: String = screen.cells().map(|c| c.c).collect();
//! assert_eq!(text.trim(), "a line of colored terminal data");
//!
//! assert_eq!(&format!("{}", screen.get(0, 0).unwrap().fg), "#839496");
//! assert_eq!(&format!("{}", screen.get(0, 10).unwrap().fg), "#859900");
//!
//! // Render the screen to SVG.
//! println!("{}", screen.to_svg(&[], FontMetrics::DEFAULT));
//! ```

#![forbid(unsafe_code)]
use std::fmt::{Display, Write};

use alacritty_terminal::{
    term::{
        cell::{Cell as AlacrittyCell, Flags},
        test::TermSize,
        Config, Term as AlacrittyTerm,
    },
    vte::{self, ansi::Processor},
};

mod ansi;
mod colors;

pub use ansi::AnsiSignal;
use colors::Colors;

/// A sensible default font size, in case some renderers don't automatically scale up the SVG.
const FONT_SIZE_PX: f32 = 12.;

/// Metrics for rendering a monospaced font.
#[derive(Clone, Copy, Debug)]
pub struct FontMetrics {
    /// The number of font units per Em. To scale the font to a specific size, the font metrics are
    /// scaled relative to this unit. For example, the line height in pixels for a font at size
    /// 12px would be:
    ///
    /// `line_height / units_per_em * 12`
    pub units_per_em: u16,
    /// The amount of horizontal advance between characters.
    pub advance: f32,
    /// Height between the baselines of two lines of text.
    pub line_height: f32,
    /// Space below the text baseline. This is the distance between the text baseline of a line
    /// and the top of the next line.
    pub descent: f32,
}

impl FontMetrics {
    /// Font metrics that should work for fonts that are similar to, e.g., Liberation mono, Consolas
    /// or Menlo. If this is not accurate, it will be noticeable as overlap or gaps between box
    /// drawing characters.
    ///
    /// ```norun
    /// FontMetrics {
    ///     units_per_em: 1000,
    ///     advance: 600.0,
    ///     line_height: 1200.0,
    ///     descent: 300.0,
    /// }
    /// ```
    pub const DEFAULT: FontMetrics = FontMetrics {
        units_per_em: 1000,
        advance: 600.,
        line_height: 1200.,
        descent: 300.,

        // Metrics of some fonts:
        // - Liberation mono:
        //     units_per_em: 2048,  1.000
        //     advance: 1229.,      0.600
        //     line_height: 2320.,  1.133
        //     descent: 615.,       0.300
        //
        // - Consolas:
        //     units_per_em: 2048,  1.000
        //     advance: 1226,       0.599
        //     line_height: 2398,   1.171
        //     descent: 514,        0.251
        //
        // - Menlo:
        //     units_per_em: 2048,  1.000
        //     advance: 1233,       0.602
        //     line_height: 2384,   1.164
        //     descent: 483,        0.236
        //
        // - Source Code Pro
        //     units_per_em: 1000,  1.000
        //     advance: 600.,       0.600
        //     line_height: 1257.,  1.257
        //     descent: 273.,       0.273

        // - Iosevka extended
        //     units_per_em: 1000,  1.000
        //     advance: 600.,       0.600
        //     line_height: 1250.,  1.250
        //     descent: 285.,       0.285
    };
}

impl Default for FontMetrics {
    fn default() -> Self {
        FontMetrics::DEFAULT
    }
}

/// Metrics for a font at a specific font size. Calculated from [FontMetrics].
#[derive(Clone, Copy)]
struct CalculatedFontMetrics {
    /// The amount of horizontal advance between characters.
    advance: f32,
    /// Height of a line of text. Lines of text directly touch each other, i.e., it is assumed
    /// the text "leading" is 0.
    line_height: f32,
    /// Distance below the text baseline. This is the distance between the text baseline of a line
    /// and the top of the next line.It is assumed there is no
    descent: f32,
}

impl FontMetrics {
    /// Get the font metrics at a specific font size.
    fn at_font_size(self, font_size: f32) -> CalculatedFontMetrics {
        let scale_factor = font_size / f32::from(self.units_per_em);
        CalculatedFontMetrics {
            advance: self.advance * scale_factor,
            line_height: self.line_height * scale_factor,
            descent: self.descent * scale_factor,
        }
    }
}

/// A color in the sRGB color space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Display for Rgb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x?}{:02x?}{:02x}", self.r, self.g, self.b)
    }
}

/// The unicode character and style of a single cell in the terminal grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub c: char,
    pub fg: Rgb,
    pub bg: Rgb,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}

impl Cell {
    fn from_alacritty_cell(colors: &Colors, cell: &AlacrittyCell) -> Self {
        Cell {
            c: cell.c,
            fg: colors.to_rgb(cell.fg),
            bg: colors.to_rgb(cell.bg),
            bold: cell.flags.intersects(Flags::BOLD),
            italic: cell.flags.intersects(Flags::ITALIC),
            underline: cell.flags.intersects(Flags::ALL_UNDERLINES),
            strikethrough: cell.flags.intersects(Flags::STRIKEOUT),
        }
    }
}

#[derive(PartialEq)]
struct TextStyle {
    fg: Rgb,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

impl TextStyle {
    /// private conversion from alacritty Cell to Style
    fn from_cell(cell: &Cell) -> Self {
        let Cell {
            fg,
            bold,
            italic,
            underline,
            strikethrough,
            ..
        } = *cell;

        TextStyle {
            fg,
            bold,
            italic,
            underline,
            strikethrough,
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
    color: Rgb,
    font_metrics: &CalculatedFontMetrics,
) -> std::fmt::Result {
    writeln!(
        f,
        r#"<rect x="{x}" y="{y}" width="{width}" height="{height}" style="fill: {color};" />"#,
        x = f32::from(x0) * font_metrics.advance,
        y = f32::from(y0) * font_metrics.line_height,
        width = f32::from(x1 - x0 + 1) * font_metrics.advance,
        height = f32::from(y1 - y0 + 1) * font_metrics.line_height,
        color = color,
    )
}

fn fmt_text(
    f: &mut std::fmt::Formatter<'_>,
    x: u16,
    y: u16,
    text: &TextLine,
    style: &TextStyle,
    font_metrics: &CalculatedFontMetrics,
) -> std::fmt::Result {
    let chars = text.chars();
    let text_length = chars.len() as f32 * font_metrics.advance;
    write!(
        f,
        r#"<text x="{x}" y="{y}" textLength="{text_length}" style="fill: {color};"#,
        x = f32::from(x) * font_metrics.advance,
        y = f32::from(y + 1) * font_metrics.line_height - font_metrics.descent,
        color = style.fg,
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
    let mut prev_char_was_space = false;
    for char in chars {
        match *char {
            ' ' => {
                if prev_char_was_space {
                    // non-breaking space
                    f.write_str("&#160;")?
                } else {
                    f.write_char(' ')?
                }
            }
            // escape tag opening
            '<' => f.write_str("&lt;")?,
            '&' => f.write_str("&amp;")?,
            c => f.write_char(c)?,
        }

        prev_char_was_space = *char == ' ';
    }
    f.write_str("</text>\n")?;

    Ok(())
}

/// A static snapshot of a terminal screen.
pub struct Screen {
    lines: u16,
    columns: u16,
    cells: Vec<Cell>,
}

impl Screen {
    /// Get a [std::fmt::Display] that prints an SVG when formatted. Set `fonts` to specify fonts
    /// to be included in the SVG's `font-family` style. `font-family` always includes `monospace`.
    ///
    /// The SVG is generated once [std::fmt::Display::fmt] is called; cache the call's output if
    /// you want to use it multiple times.
    pub fn to_svg<'s, 'f>(
        &'s self,
        fonts: &'f [&'f str],
        font_metrics: FontMetrics,
    ) -> impl Display + 's
    where
        'f: 's,
    {
        struct Svg<'s> {
            screen: &'s Screen,
            fonts: &'s [&'s str],
            font_metrics: CalculatedFontMetrics,
        }

        impl<'s> Display for Svg<'s> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let font_metrics = self.font_metrics;

                let Screen {
                    lines,
                    columns,
                    ref cells,
                } = self.screen;

                write!(
                    f,
                    r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
                    f32::from(*columns) * font_metrics.advance,
                    f32::from(*lines) * font_metrics.line_height,
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

                write!(
                    f,
                    r#"monospace;
    font-size: {FONT_SIZE_PX}px;
  }}
</style>
<g class="screen">
"#,
                )?;

                let main_bg = colors::most_common_color(self.screen);
                fmt_rect(
                    f,
                    0,
                    0,
                    self.screen.columns().saturating_sub(1),
                    self.screen.lines().saturating_sub(1),
                    main_bg,
                    &font_metrics,
                )?;

                // find background rectangles to draw by greedily flooding lines then flooding down columns
                let mut drawn = vec![false; usize::from(*lines) * usize::from(*columns)];
                for y0 in 0..*lines {
                    for x0 in 0..*columns {
                        let idx = self.screen.idx(y0, x0);

                        if drawn[idx] {
                            continue;
                        }

                        let cell = &cells[idx];
                        let bg = cell.bg;

                        if bg == main_bg {
                            continue;
                        }

                        let mut end_x = x0;
                        let mut end_y = y0;

                        for x1 in x0 + 1..*columns {
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
                            for x1 in x0 + 1..*columns {
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

                        fmt_rect(f, x0, y0, end_x, end_y, bg, &font_metrics)?;
                    }
                }

                // write text
                let mut text_line =
                    TextLine::with_capacity(usize::from(*columns).next_power_of_two());
                for y in 0..*lines {
                    let idx = self.screen.idx(y, 0);
                    let cell = &cells[idx];
                    let mut style = TextStyle::from_cell(cell);
                    let mut start_x = 0;

                    for x in 0..*columns {
                        let idx = self.screen.idx(y, x);
                        let cell = &cells[idx];
                        let style_ = TextStyle::from_cell(cell);

                        if style_ != style {
                            if !text_line.is_empty() {
                                fmt_text(f, start_x, y, &text_line, &style, &font_metrics)?;
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
                        fmt_text(f, start_x, y, &text_line, &style, &font_metrics)?;
                        text_line.clear();
                    }
                }

                f.write_str(
                    "</g>
</svg>",
                )?;

                Ok(())
            }
        }

        Svg {
            screen: self,
            fonts,
            font_metrics: font_metrics.at_font_size(FONT_SIZE_PX),
        }
    }

    #[inline(always)]
    fn idx(&self, y: u16, x: u16) -> usize {
        usize::from(y) * usize::from(self.columns) + usize::from(x)
    }

    /// The number of screen lines in this snapshot.
    pub fn lines(&self) -> u16 {
        self.lines
    }

    /// The number of screen columns in this snapshot.
    pub fn columns(&self) -> u16 {
        self.columns
    }

    /// An iterator over all cells in the terminal grid. This iterates over all columns in the
    /// first line from left to right, then the second line, etc.
    pub fn cells(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter()
    }

    /// Get the cell at the terminal grid position specified by `line` and `column`.
    pub fn get(&self, line: u16, column: u16) -> Option<&Cell> {
        self.cells.get(self.idx(line, column))
    }
}

/// A sink for responses sent by the [terminal emulator](Term). The terminal emulator sends
/// responses to ANSI requests. Implement this trait to process these responses, e.g., by sending
/// them to the requesting pseudoterminal.
pub trait PtyWriter {
    /// Write `text` on the terminal in response to an ANSI request.
    fn write(&mut self, text: String);
}

impl<F: FnMut(String)> PtyWriter for F {
    fn write(&mut self, text: String) {
        self(text)
    }
}

/// A [`PtyWriter`] that ignores all responses.
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
pub struct Term<W: PtyWriter> {
    lines: u16,
    columns: u16,
    term: AlacrittyTerm<EventProxy<W>>,
    processor: Option<vte::ansi::Processor<vte::ansi::StdSyncHandler>>,
}

impl<W: PtyWriter> Term<W> {
    /// Create a new emulated terminal with a cell matrix of `lines` by `columns`.
    ///
    /// [`pty_writer`](PtyWriter) is used to send output from the emulated terminal in reponse to ANSI requests.
    /// Use [`VoidPtyWriter`] if you do not need to send responses to status requests.
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
            processor: Some(Processor::new()),
        }
    }

    /// Process one byte of ANSI-escaped terminal data.
    pub fn process(&mut self, byte: u8) {
        self.processor
            .as_mut()
            .expect("unreachable")
            .advance(&mut self.term, byte);
    }

    /// Process one byte of ANSI-escaped terminal data. Some ANSI signals will trigger callback
    /// `cb` to be called with a reference to the terminal and the signal that triggered the call,
    /// right before applying the result of the ANSI signal to the terminal. This allows grabbing a
    /// snapshot of the the terminal screen contents before application of the signal.
    ///
    /// See also [AnsiSignal].
    pub fn process_with_callback(&mut self, byte: u8, mut cb: impl FnMut(&Self, AnsiSignal)) {
        let mut processor = self.processor.take().expect("unreachable");

        let mut handler = ansi::HandlerWrapper {
            term: self,
            cb: &mut cb,
        };

        processor.advance(&mut handler, byte);
        self.processor = Some(processor);
    }

    /// Resize the terminal screen to the specified dimension.
    pub fn resize(&mut self, lines: u16, columns: u16) {
        let new_size = TermSize {
            columns: columns.into(),
            screen_lines: lines.into(),
        };
        self.lines = lines;
        self.columns = columns;
        self.term.resize(new_size);
    }

    /// Get a snapshot of the current terminal screen.
    pub fn current_screen(&self) -> Screen {
        // ideally users can define their own colors
        let colors = Colors::default();

        Screen {
            lines: self.lines,
            columns: self.columns,
            cells: self
                .term
                .grid()
                .display_iter()
                .map(|point_cell| Cell::from_alacritty_cell(&colors, point_cell.cell))
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
