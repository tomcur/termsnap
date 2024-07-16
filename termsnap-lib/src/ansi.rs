use alacritty_terminal::vte::ansi::{self, Handler};

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

impl<'t, W: PtyWriter> Handler for HandlerWrapper<'t, W> {
    fn set_title(&mut self, p: Option<String>) {
        self.term.term.set_title(p)
    }
    fn set_cursor_style(&mut self, p: Option<ansi::CursorStyle>) {
        self.term.term.set_cursor_style(p)
    }
    fn set_cursor_shape(&mut self, p: ansi::CursorShape) {
        self.term.term.set_cursor_shape(p)
    }
    fn input(&mut self, p: char) {
        self.term.term.input(p)
    }
    fn goto(&mut self, p1: i32, p2: usize) {
        self.term.term.goto(p1, p2)
    }
    fn goto_line(&mut self, p: i32) {
        self.term.term.goto_line(p)
    }
    fn goto_col(&mut self, p: usize) {
        self.term.term.goto_col(p)
    }
    fn insert_blank(&mut self, p: usize) {
        self.term.term.insert_blank(p)
    }
    fn move_up(&mut self, p: usize) {
        self.term.term.move_up(p)
    }
    fn move_down(&mut self, p: usize) {
        self.term.term.move_down(p)
    }
    fn identify_terminal(&mut self, p: Option<char>) {
        self.term.term.identify_terminal(p)
    }
    fn device_status(&mut self, p: usize) {
        self.term.term.device_status(p)
    }
    fn move_forward(&mut self, p: usize) {
        self.term.term.move_forward(p)
    }
    fn move_backward(&mut self, p: usize) {
        self.term.term.move_backward(p)
    }
    fn move_down_and_cr(&mut self, p: usize) {
        self.term.term.move_down_and_cr(p)
    }
    fn move_up_and_cr(&mut self, p: usize) {
        self.term.term.move_up_and_cr(p)
    }
    fn put_tab(&mut self, p: u16) {
        self.term.term.put_tab(p)
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
    fn substitute(&mut self) {
        self.term.term.substitute()
    }
    fn newline(&mut self) {
        self.term.term.newline()
    }
    fn set_horizontal_tabstop(&mut self) {
        self.term.term.set_horizontal_tabstop()
    }
    fn scroll_up(&mut self, p: usize) {
        self.term.term.scroll_up(p)
    }
    fn scroll_down(&mut self, p: usize) {
        self.term.term.scroll_down(p)
    }
    fn insert_blank_lines(&mut self, p: usize) {
        self.term.term.insert_blank_lines(p)
    }
    fn delete_lines(&mut self, p: usize) {
        self.term.term.delete_lines(p)
    }
    fn erase_chars(&mut self, p: usize) {
        self.term.term.erase_chars(p)
    }
    fn delete_chars(&mut self, p: usize) {
        self.term.term.delete_chars(p)
    }
    fn move_backward_tabs(&mut self, p: u16) {
        self.term.term.move_backward_tabs(p)
    }
    fn move_forward_tabs(&mut self, p: u16) {
        self.term.term.move_forward_tabs(p)
    }
    fn save_cursor_position(&mut self) {
        self.term.term.save_cursor_position()
    }
    fn restore_cursor_position(&mut self) {
        self.term.term.restore_cursor_position()
    }
    fn clear_line(&mut self, p: ansi::LineClearMode) {
        self.term.term.clear_line(p)
    }
    fn clear_screen(&mut self, p: ansi::ClearMode) {
        (self.cb)(&self.term, AnsiSignal::ClearScreen);

        self.term.term.clear_screen(p)
    }
    fn clear_tabs(&mut self, p: ansi::TabulationClearMode) {
        self.term.term.clear_tabs(p)
    }
    fn reset_state(&mut self) {
        self.term.term.reset_state()
    }
    fn reverse_index(&mut self) {
        self.term.term.reverse_index()
    }
    fn terminal_attribute(&mut self, p: ansi::Attr) {
        self.term.term.terminal_attribute(p)
    }
    fn set_mode(&mut self, p: ansi::Mode) {
        self.term.term.set_mode(p)
    }
    fn unset_mode(&mut self, p: ansi::Mode) {
        self.term.term.unset_mode(p)
    }
    fn report_mode(&mut self, p: ansi::Mode) {
        self.term.term.report_mode(p)
    }
    fn set_private_mode(&mut self, p: ansi::PrivateMode) {
        if matches!(
            p,
            ansi::PrivateMode::Named(ansi::NamedPrivateMode::SwapScreenAndSetRestoreCursor)
        ) {
            (self.cb)(
                &self.term,
                AnsiSignal::AlternateScreenBuffer { enable: true },
            );
        }

        self.term.term.set_private_mode(p)
    }
    fn unset_private_mode(&mut self, p: ansi::PrivateMode) {
        if matches!(
            p,
            ansi::PrivateMode::Named(ansi::NamedPrivateMode::SwapScreenAndSetRestoreCursor)
        ) {
            (self.cb)(
                &self.term,
                AnsiSignal::AlternateScreenBuffer { enable: false },
            );
        }

        self.term.term.unset_private_mode(p)
    }
    fn report_private_mode(&mut self, p: ansi::PrivateMode) {
        self.term.term.report_private_mode(p)
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
    fn set_active_charset(&mut self, p: ansi::CharsetIndex) {
        self.term.term.set_active_charset(p)
    }
    fn configure_charset(&mut self, p1: ansi::CharsetIndex, p2: ansi::StandardCharset) {
        self.term.term.configure_charset(p1, p2)
    }
    fn set_color(&mut self, p1: usize, p2: ansi::Rgb) {
        self.term.term.set_color(p1, p2)
    }
    fn dynamic_color_sequence(&mut self, p1: String, p2: usize, p3: &str) {
        self.term.term.dynamic_color_sequence(p1, p2, p3)
    }
    fn reset_color(&mut self, p: usize) {
        self.term.term.reset_color(p)
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
    fn text_area_size_chars(&mut self) {
        self.term.term.text_area_size_chars()
    }
    fn set_hyperlink(&mut self, p: Option<ansi::Hyperlink>) {
        self.term.term.set_hyperlink(p)
    }
    fn set_mouse_cursor_icon(&mut self, p: ansi::cursor_icon::CursorIcon) {
        self.term.term.set_mouse_cursor_icon(p)
    }
    fn report_keyboard_mode(&mut self) {
        self.term.term.report_keyboard_mode()
    }
    fn push_keyboard_mode(&mut self, p: ansi::KeyboardModes) {
        self.term.term.push_keyboard_mode(p)
    }
    fn pop_keyboard_modes(&mut self, p: u16) {
        self.term.term.pop_keyboard_modes(p)
    }
    fn set_keyboard_mode(&mut self, p1: ansi::KeyboardModes, p2: ansi::KeyboardModesApplyBehavior) {
        self.term.term.set_keyboard_mode(p1, p2)
    }
    fn set_modify_other_keys(&mut self, p: ansi::ModifyOtherKeys) {
        self.term.term.set_modify_other_keys(p)
    }
    fn report_modify_other_keys(&mut self) {
        self.term.term.report_modify_other_keys()
    }
}
