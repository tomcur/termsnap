use rio_vt::ansi::mode::{Mode, NamedPrivateMode, PrivateMode};
use rio_vt::ansi::{
    glyph_protocol, kitty_graphics_protocol, sixel, ClearMode, CursorShape, KeyboardModes,
    KeyboardModesApplyBehavior, LineClearMode, TabulationClearMode,
};
use rio_vt::config::colors::ColorRgb;
use rio_vt::crosswords::attr::Attr;
use rio_vt::crosswords::grid::row::SemanticPrompt;
use rio_vt::crosswords::pos::{CharsetIndex, Column, Line, StandardCharset};
use rio_vt::crosswords::square::Hyperlink;
use rio_vt::event::ProgressReport;
use rio_vt::performer::handler::{Handler, ScpCharPath, ScpUpdateMode};
use rio_vt::performer::parser::Params;

use crate::{PtyWriter, Term};

pub enum AnsiSignal {
    /// Clear the entire terminal screen.
    ClearScreen,
    /// Enable or disable the alternate terminal screen buffer.
    AlternateScreenBuffer { enable: bool },
}

pub(crate) struct HandlerWrapper<'t, W: PtyWriter> {
    pub term: &'t mut Term<W>,
    pub cb: &'t mut dyn FnMut(&Term<W>, AnsiSignal),
}

/// Forwards the full [`Handler`] surface to the wrapped terminal, intercepting
/// the signals exposed as [`AnsiSignal`]. Methods whose signatures use types
/// from crates rio-vt does not re-export (image insertion, mouse cursor icon)
/// are left at their default no-op; termsnap does not render those anyway.
impl<'t, W: PtyWriter> Handler for HandlerWrapper<'t, W> {
    fn set_title(&mut self, p1: Option<String>) {
        self.term.term.set_title(p1)
    }
    fn set_current_directory(&mut self, p1: std::path::PathBuf) {
        self.term.term.set_current_directory(p1)
    }
    fn set_semantic_prompt(&mut self, p1: SemanticPrompt) {
        self.term.term.set_semantic_prompt(p1)
    }
    fn set_user_var(&mut self, p1: String, p2: String) {
        self.term.term.set_user_var(p1, p2)
    }
    fn set_cursor_style(&mut self, p1: Option<CursorShape>, p2: bool) {
        self.term.term.set_cursor_style(p1, p2)
    }
    fn set_cursor_shape(&mut self, p1: CursorShape) {
        self.term.term.set_cursor_shape(p1)
    }
    fn input(&mut self, p1: char) {
        self.term.term.input(p1)
    }
    fn input_str(&mut self, p1: &str) {
        self.term.term.input_str(p1)
    }
    fn input_ascii_str(&mut self, p1: &str) {
        self.term.term.input_ascii_str(p1)
    }
    fn input_codepoints(&mut self, p1: &[u32]) {
        self.term.term.input_codepoints(p1)
    }
    fn goto(&mut self, p1: Line, p2: Column) {
        self.term.term.goto(p1, p2)
    }
    fn goto_line(&mut self, p1: Line) {
        self.term.term.goto_line(p1)
    }
    fn goto_col(&mut self, p1: Column) {
        self.term.term.goto_col(p1)
    }
    fn insert_blank(&mut self, p1: usize) {
        self.term.term.insert_blank(p1)
    }
    fn move_up(&mut self, p1: usize) {
        self.term.term.move_up(p1)
    }
    fn move_down(&mut self, p1: usize) {
        self.term.term.move_down(p1)
    }
    fn identify_terminal(&mut self, p1: Option<char>) {
        self.term.term.identify_terminal(p1)
    }
    fn device_status(&mut self, p1: usize) {
        self.term.term.device_status(p1)
    }
    fn move_forward(&mut self, p1: Column) {
        self.term.term.move_forward(p1)
    }
    fn move_backward(&mut self, p1: Column) {
        self.term.term.move_backward(p1)
    }
    fn move_down_and_cr(&mut self, p1: usize) {
        self.term.term.move_down_and_cr(p1)
    }
    fn move_up_and_cr(&mut self, p1: usize) {
        self.term.term.move_up_and_cr(p1)
    }
    fn put_tab(&mut self, p1: u16) {
        self.term.term.put_tab(p1)
    }
    fn backspace(&mut self) {
        self.term.term.backspace()
    }
    fn carriage_return(&mut self) {
        self.term.term.carriage_return()
    }
    fn linefeed(&mut self) {
        self.term.term.linefeed()
    }
    fn bell(&mut self) {
        self.term.term.bell()
    }
    fn desktop_notification(&mut self, p1: String, p2: String) {
        self.term.term.desktop_notification(p1, p2)
    }
    fn substitute(&mut self) {
        self.term.term.substitute()
    }
    fn newline(&mut self) {
        self.term.term.newline()
    }
    fn set_horizontal_tabstop(&mut self) {
        self.term.term.set_horizontal_tabstop()
    }
    fn scroll_up(&mut self, p1: usize) {
        self.term.term.scroll_up(p1)
    }
    fn scroll_down(&mut self, p1: usize) {
        self.term.term.scroll_down(p1)
    }
    fn insert_blank_lines(&mut self, p1: usize) {
        self.term.term.insert_blank_lines(p1)
    }
    fn delete_lines(&mut self, p1: usize) {
        self.term.term.delete_lines(p1)
    }
    fn erase_chars(&mut self, p1: Column) {
        self.term.term.erase_chars(p1)
    }
    fn delete_chars(&mut self, p1: usize) {
        self.term.term.delete_chars(p1)
    }
    fn move_backward_tabs(&mut self, p1: u16) {
        self.term.term.move_backward_tabs(p1)
    }
    fn move_forward_tabs(&mut self, p1: u16) {
        self.term.term.move_forward_tabs(p1)
    }
    fn save_cursor_position(&mut self) {
        self.term.term.save_cursor_position()
    }
    fn restore_cursor_position(&mut self) {
        self.term.term.restore_cursor_position()
    }
    fn clear_line(&mut self, p1: LineClearMode) {
        self.term.term.clear_line(p1)
    }
    fn clear_screen(&mut self, p1: ClearMode) {
        (self.cb)(self.term, AnsiSignal::ClearScreen);

        self.term.term.clear_screen(p1)
    }
    fn set_tabs(&mut self, p1: u16) {
        self.term.term.set_tabs(p1)
    }
    fn clear_tabs(&mut self, p1: TabulationClearMode) {
        self.term.term.clear_tabs(p1)
    }
    fn reset_state(&mut self) {
        self.term.term.reset_state()
    }
    fn reverse_index(&mut self) {
        self.term.term.reverse_index()
    }
    fn terminal_attribute(&mut self, p1: Attr) {
        self.term.term.terminal_attribute(p1)
    }
    fn set_mode(&mut self, p1: Mode) {
        self.term.term.set_mode(p1)
    }
    fn unset_mode(&mut self, p1: Mode) {
        self.term.term.unset_mode(p1)
    }
    fn report_mode(&mut self, p1: Mode) {
        self.term.term.report_mode(p1)
    }
    fn set_private_mode(&mut self, p1: PrivateMode) {
        if matches!(
            p1,
            PrivateMode::Named(NamedPrivateMode::SwapScreenAndSetRestoreCursor)
        ) {
            (self.cb)(
                self.term,
                AnsiSignal::AlternateScreenBuffer { enable: true },
            );
        }

        self.term.term.set_private_mode(p1)
    }
    fn unset_private_mode(&mut self, p1: PrivateMode) {
        if matches!(
            p1,
            PrivateMode::Named(NamedPrivateMode::SwapScreenAndSetRestoreCursor)
        ) {
            (self.cb)(
                self.term,
                AnsiSignal::AlternateScreenBuffer { enable: false },
            );
        }

        self.term.term.unset_private_mode(p1)
    }
    fn report_private_mode(&mut self, p1: PrivateMode) {
        self.term.term.report_private_mode(p1)
    }
    fn report_version(&mut self) {
        self.term.term.report_version()
    }
    fn set_scrolling_region(&mut self, p1: usize, p2: Option<usize>) {
        self.term.term.set_scrolling_region(p1, p2)
    }
    fn set_keypad_application_mode(&mut self) {
        self.term.term.set_keypad_application_mode()
    }
    fn unset_keypad_application_mode(&mut self) {
        self.term.term.unset_keypad_application_mode()
    }
    fn set_active_charset(&mut self, p1: CharsetIndex) {
        self.term.term.set_active_charset(p1)
    }
    fn configure_charset(&mut self, p1: CharsetIndex, p2: StandardCharset) {
        self.term.term.configure_charset(p1, p2)
    }
    fn set_color(&mut self, p1: usize, p2: ColorRgb) {
        self.term.term.set_color(p1, p2)
    }
    fn dynamic_color_sequence(&mut self, p1: String, p2: usize, p3: &str) {
        self.term.term.dynamic_color_sequence(p1, p2, p3)
    }
    fn reset_color(&mut self, p1: usize) {
        self.term.term.reset_color(p1)
    }
    fn clipboard_store(&mut self, p1: u8, p2: &[u8]) {
        self.term.term.clipboard_store(p1, p2)
    }
    fn clipboard_load(&mut self, p1: u8, p2: &str) {
        self.term.term.clipboard_load(p1, p2)
    }
    fn decaln(&mut self) {
        self.term.term.decaln()
    }
    fn push_title(&mut self) {
        self.term.term.push_title()
    }
    fn pop_title(&mut self) {
        self.term.term.pop_title()
    }
    fn text_area_size_pixels(&mut self) {
        self.term.term.text_area_size_pixels()
    }
    fn cells_size_pixels(&mut self) {
        self.term.term.cells_size_pixels()
    }
    fn text_area_size_chars(&mut self) {
        self.term.term.text_area_size_chars()
    }
    fn graphics_attribute(&mut self, p1: u16, p2: u16) {
        self.term.term.graphics_attribute(p1, p2)
    }
    fn sixel_graphic_start(&mut self, p1: &Params) {
        self.term.term.sixel_graphic_start(p1)
    }
    fn is_sixel_graphic_active(&self) -> bool {
        self.term.term.is_sixel_graphic_active()
    }
    fn sixel_graphic_put(&mut self, p1: u8) -> Result<(), sixel::Error> {
        self.term.term.sixel_graphic_put(p1)
    }
    fn sixel_graphic_reset(&mut self) {
        self.term.term.sixel_graphic_reset()
    }
    fn sixel_graphic_finish(&mut self) {
        self.term.term.sixel_graphic_finish()
    }
    fn place_graphic(&mut self, p1: kitty_graphics_protocol::PlacementRequest) {
        self.term.term.place_graphic(p1)
    }
    fn delete_graphics(&mut self, p1: kitty_graphics_protocol::DeleteRequest) {
        self.term.term.delete_graphics(p1)
    }
    fn set_hyperlink(&mut self, p1: Option<Hyperlink>) {
        self.term.term.set_hyperlink(p1)
    }
    fn set_progress_report(&mut self, p1: ProgressReport) {
        self.term.term.set_progress_report(p1)
    }
    fn report_keyboard_mode(&mut self) {
        self.term.term.report_keyboard_mode()
    }
    fn push_keyboard_mode(&mut self, p1: KeyboardModes) {
        self.term.term.push_keyboard_mode(p1)
    }
    fn pop_keyboard_modes(&mut self, p1: u16) {
        self.term.term.pop_keyboard_modes(p1)
    }
    fn set_keyboard_mode(&mut self, p1: KeyboardModes, p2: KeyboardModesApplyBehavior) {
        self.term.term.set_keyboard_mode(p1, p2)
    }
    fn xtgettcap_response(&mut self, p1: String) {
        self.term.term.xtgettcap_response(p1)
    }
    fn kitty_graphics_response(&mut self, p1: String) {
        self.term.term.kitty_graphics_response(p1)
    }
    fn glyph_protocol_response(&mut self, p1: String) {
        self.term.term.glyph_protocol_response(p1)
    }
    fn glyph_register(&mut self, p1: u32, p2: glyph_protocol::GlyphPayload, p3: u8) -> Result<(), glyph_protocol::RegisterError> {
        self.term.term.glyph_register(p1, p2, p3)
    }
    fn glyph_clear(&mut self, p1: Option<u32>) {
        self.term.term.glyph_clear(p1)
    }
    fn glyph_query(&mut self, p1: u32) {
        self.term.term.glyph_query(p1)
    }
    fn kitty_chunking_state_mut(&mut self) -> Option<&mut kitty_graphics_protocol::KittyGraphicsState> {
        self.term.term.kitty_chunking_state_mut()
    }
    fn set_scp(&mut self, p1: ScpCharPath, p2: ScpUpdateMode) {
        self.term.term.set_scp(p1, p2)
    }
}
