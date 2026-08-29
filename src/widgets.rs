// Copyright 2026 ZenLunarDev
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Widgets: panels, text, ASCII art, big banners, plus interactive
//! components ([`Btn`] and [`Input`]) that keep their own state.

use crate::border::{self, BorderSet};
use crate::buffer::Buffer;
use crate::canvas::Canvas;
use crate::color::Color;
use crate::event::{Event, KeyCode};
use crate::font::{self, GLYPH_W};
use crate::frame::Frame;
use crate::layout::{Align, Rect};
use crate::style::Style;

// ---------------------------------------------------------------------------
// Panel — a bordered box with an optional title.
// ---------------------------------------------------------------------------

/// A bordered panel drawn immediately into the buffer.
pub struct Panel<'a> {
    buf: &'a mut Buffer,
    rect: Rect,
    border: BorderSet,
    style: Style,
    title: Option<String>,
    title_align: Align,
}

impl<'a> Panel<'a> {
    pub fn new(buf: &'a mut Buffer, rect: Rect) -> Self {
        Panel {
            buf,
            rect,
            border: border::round(),
            style: Style::default(),
            title: None,
            title_align: Align::Left,
        }
    }

    pub fn thin(mut self) -> Self {
        self.border = border::thin();
        self
    }
    pub fn round(mut self) -> Self {
        self.border = border::round();
        self
    }
    pub fn double(mut self) -> Self {
        self.border = border::double();
        self
    }
    pub fn thick(mut self) -> Self {
        self.border = border::thick();
        self
    }
    pub fn ascii(mut self) -> Self {
        self.border = border::ascii();
        self
    }
    pub fn curve(mut self) -> Self {
        self.border = border::curve();
        self
    }
    pub fn dotted(mut self) -> Self {
        self.border = border::dotted();
        self
    }
    pub fn dashed(mut self) -> Self {
        self.border = border::dashed();
        self
    }
    pub fn block(mut self) -> Self {
        self.border = border::block();
        self
    }
    pub fn border_set(mut self, b: BorderSet) -> Self {
        self.border = b;
        self
    }

    pub fn fg(mut self, c: Color) -> Self {
        self.style.fg = c;
        self
    }
    pub fn bg(mut self, c: Color) -> Self {
        self.style.bg = c;
        self
    }
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn title_align(mut self, a: Align) -> Self {
        self.title_align = a;
        self
    }

    /// Draw the panel (and title) into the buffer.
    pub fn draw(self) {
        let mut c = Canvas::new(self.buf);
        c.border(self.rect, self.border, self.style);
        if let Some(t) = &self.title {
            let inner = self.rect.inner();
            let off = self.title_align.h_offset(t.chars().count() as u16, inner.w);
            let x = inner.x as i32 + off as i32;
            c.text(x, self.rect.y as i32, t, Align::Left, t.len() as u16, self.style);
        }
    }
}

// ---------------------------------------------------------------------------
// Text — multiline, aligned, optionally wrapped.
// ---------------------------------------------------------------------------

/// Immediate text drawing into a rectangular region.
pub struct Text<'a> {
    buf: &'a mut Buffer,
    rect: Rect,
    style: Style,
    align: Align,
    wrap: bool,
}

impl<'a> Text<'a> {
    pub fn new(buf: &'a mut Buffer, rect: Rect) -> Self {
        Text {
            buf,
            rect,
            style: Style::default(),
            align: Align::Left,
            wrap: false,
        }
    }

    pub fn fg(mut self, c: Color) -> Self {
        self.style.fg = c;
        self
    }
    pub fn bg(mut self, c: Color) -> Self {
        self.style.bg = c;
        self
    }
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
    pub fn align(mut self, a: Align) -> Self {
        self.align = a;
        self
    }
    pub fn wrap(mut self, v: bool) -> Self {
        self.wrap = v;
        self
    }

    /// Draw `content` into the region.
    pub fn draw(self, content: &str) {
        let mut c = Canvas::new(self.buf);
        let mut row = self.rect.y as i32;
        for line in content.split('\n') {
            let line = if self.wrap {
                wrap_line(line, self.rect.w as usize)
            } else {
                vec![line.to_string()]
            };
            for sub in line {
                if row >= self.rect.bottom() as i32 {
                    return;
                }
                c.text(
                    self.rect.x as i32,
                    row,
                    &sub,
                    self.align,
                    self.rect.w,
                    self.style,
                );
                row += 1;
            }
        }
    }
}

fn wrap_line(s: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![s.to_string()];
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    for word in s.split(' ') {
        if cur.is_empty() {
            cur = word.to_string();
        } else if cur.chars().count() + 1 + word.chars().count() <= width {
            cur.push(' ');
            cur.push_str(word);
        } else {
            out.push(std::mem::take(&mut cur));
            cur = word.to_string();
        }
        while cur.chars().count() > width {
            let split: String = cur.chars().take(width).collect();
            out.push(split);
            cur = cur.chars().skip(width).collect();
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

// ---------------------------------------------------------------------------
// Ascii — render pre-made ASCII art verbatim.
// ---------------------------------------------------------------------------

/// Draw a multi-line ASCII art block at `(x, y)`.
pub struct Ascii<'a> {
    buf: &'a mut Buffer,
    x: u16,
    y: u16,
    style: Style,
}

impl<'a> Ascii<'a> {
    pub fn new(buf: &'a mut Buffer, x: u16, y: u16) -> Self {
        Ascii {
            buf,
            x,
            y,
            style: Style::default(),
        }
    }
    pub fn fg(mut self, c: Color) -> Self {
        self.style.fg = c;
        self
    }
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
    /// Draw `art` (may contain newlines and spaces) preserving layout.
    pub fn draw(self, art: &str) {
        let mut c = Canvas::new(self.buf);
        let mut yy = self.y as i32;
        for line in art.split('\n') {
            c.text(self.x as i32, yy, line, Align::Left, line.len() as u16, self.style);
            yy += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Banner — big text using the built-in font.
// ---------------------------------------------------------------------------

/// Render text using the built-in 5-row banner font.
pub struct Banner<'a> {
    buf: &'a mut Buffer,
    x: u16,
    y: u16,
    style: Style,
    spacing: u16,
}

impl<'a> Banner<'a> {
    pub fn new(buf: &'a mut Buffer, x: u16, y: u16) -> Self {
        Banner {
            buf,
            x,
            y,
            style: Style::default(),
            spacing: 1,
        }
    }
    pub fn fg(mut self, c: Color) -> Self {
        self.style.fg = c;
        self
    }
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
    /// Extra cells between glyphs.
    pub fn spacing(mut self, n: u16) -> Self {
        self.spacing = n;
        self
    }

    /// Draw `text` big.
    pub fn draw(self, text: &str) {
        let mut c = Canvas::new(self.buf);
        let mut cx = self.x as i32;
        for ch in text.chars() {
            let g = font::glyph(ch).unwrap_or(["     "; 5]);
            for (row, line) in g.iter().enumerate() {
                let yy = self.y as i32 + row as i32;
                for (col, gc) in line.chars().enumerate() {
                    if gc != ' ' && gc != '\0' {
                        c.put(cx + col as i32, yy, gc, self.style);
                    }
                }
            }
            cx += GLYPH_W as i32 + self.spacing as i32;
        }
    }

    /// Width in cells the given text will occupy.
    pub fn width_of(text: &str) -> u16 {
        (text.chars().count() as u16) * (GLYPH_W + 1).saturating_sub(0)
    }
}

// ---------------------------------------------------------------------------
// Btn — interactive button with state.
// ---------------------------------------------------------------------------

/// A clickable button that keeps its own state across frames.
pub struct Btn {
    rect: Rect,
    label: String,
    style: Style,
    focus_style: Style,
    border: BorderSet,
    on_click: Option<Box<dyn FnMut()>>,
    id: usize,
    focused: bool,
    clicked_flag: bool,
}

impl Btn {
    pub fn new(rect: Rect) -> Self {
        Btn {
            rect,
            label: String::new(),
            style: Style::default().fg(Color::White),
            focus_style: Style::default().reverse(true).fg(Color::White),
            border: border::round(),
            on_click: None,
            id: usize::MAX,
            focused: false,
            clicked_flag: false,
        }
    }

    pub fn label(mut self, s: impl Into<String>) -> Self {
        self.label = s.into();
        self
    }
    pub fn round(mut self) -> Self {
        self.border = border::round();
        self
    }
    pub fn thin(mut self) -> Self {
        self.border = border::thin();
        self
    }
    pub fn thick(mut self) -> Self {
        self.border = border::thick();
        self
    }
    pub fn ascii(mut self) -> Self {
        self.border = border::ascii();
        self
    }
    pub fn fg(mut self, c: Color) -> Self {
        self.style.fg = c;
        self
    }
    pub fn bg(mut self, c: Color) -> Self {
        self.style.bg = c;
        self
    }
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
    pub fn focus_style(mut self, s: Style) -> Self {
        self.focus_style = s;
        self
    }
    pub fn on_click<F: FnMut() + 'static>(mut self, f: F) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Reposition the button (e.g. after a resize).
    pub fn set_rect(&mut self, r: Rect) {
        self.rect = r;
    }

    /// True on the frame the button was activated.
    pub fn clicked(&self) -> bool {
        self.clicked_flag
    }

    fn trigger(&mut self) {
        self.clicked_flag = true;
        if let Some(f) = self.on_click.as_mut() {
            f();
        }
    }

    /// Handle an event. Call before [`Btn::draw`].
    pub fn handle(&mut self, ev: &Event, frame: &mut Frame) {
        self.clicked_flag = false;
        if let Event::Key(k) = ev {
            if self.focused && (k.is_enter() || k.is_char(' ')) {
                self.trigger();
            }
        } else if let Event::Mouse(m) = ev {
            if m.kind == crate::event::MouseKind::Press
                && m.button == crate::event::MouseButton::Left
                && self.rect.contains(m.x, m.y)
            {
                self.focused = true;
                frame.request_focus(self.id);
                self.trigger();
            }
        }
    }

    /// Draw the button.
    pub fn draw(&mut self, frame: &mut Frame) {
        let is_focused = frame.register_focus();
        self.id = frame.focus_count() - 1;
        self.focused = is_focused;
        let st = if is_focused {
            self.focus_style
        } else {
            self.style
        };
        let mut c = Canvas::new(frame.buf);
        c.border(self.rect, self.border, st);
        let inner = self.rect.inner();
        let off = Align::Center.h_offset(self.label.chars().count() as u16, inner.w);
        c.text(
            inner.x as i32 + off as i32,
            (self.rect.y + self.rect.h / 2) as i32,
            &self.label,
            Align::Left,
            self.label.len() as u16,
            st,
        );
    }
}

// ---------------------------------------------------------------------------
// Input — single-line text field with editing.
// ---------------------------------------------------------------------------

/// A single-line text input field with cursor, hint and masking.
pub struct Input {
    rect: Rect,
    value: String,
    cursor: usize,
    hint: String,
    mask: Option<char>,
    style: Style,
    focus_style: Style,
    on_submit: Option<Box<dyn FnMut(&str)>>,
    on_change: Option<Box<dyn FnMut(&str)>>,
    max: Option<usize>,
    id: usize,
    focused: bool,
    submitted_flag: bool,
}

impl Input {
    pub fn new(rect: Rect) -> Self {
        Input {
            rect,
            value: String::new(),
            cursor: 0,
            hint: String::new(),
            mask: None,
            style: Style::default(),
            focus_style: Style::default().reverse(true),
            on_submit: None,
            on_change: None,
            max: None,
            id: usize::MAX,
            focused: false,
            submitted_flag: false,
        }
    }

    pub fn hint(mut self, s: impl Into<String>) -> Self {
        self.hint = s.into();
        self
    }
    pub fn mask(mut self, c: char) -> Self {
        self.mask = Some(c);
        self
    }
    pub fn fg(mut self, c: Color) -> Self {
        self.style.fg = c;
        self
    }
    pub fn bg(mut self, c: Color) -> Self {
        self.style.bg = c;
        self
    }
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
    pub fn focus_style(mut self, s: Style) -> Self {
        self.focus_style = s;
        self
    }
    pub fn max(mut self, n: usize) -> Self {
        self.max = Some(n);
        self
    }
    pub fn on_submit<F: FnMut(&str) + 'static>(mut self, f: F) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }
    pub fn on_change<F: FnMut(&str) + 'static>(mut self, f: F) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn set_value(&mut self, s: &str) {
        self.value = s.to_string();
        self.cursor = self.value.chars().count();
    }
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }
    pub fn is_focused(&self) -> bool {
        self.focused
    }
    /// Reposition the field (e.g. after a resize).
    pub fn set_rect(&mut self, r: Rect) {
        self.rect = r;
    }
    /// True on the frame Enter was pressed.
    pub fn submitted(&self) -> bool {
        self.submitted_flag
    }
    pub fn focus(&mut self) {
        self.focused = true;
    }

    /// Handle an event. Call before [`Input::draw`].
    pub fn handle(&mut self, ev: &Event, frame: &mut Frame) {
        self.submitted_flag = false;
        match ev {
            Event::Mouse(m) if m.kind == crate::event::MouseKind::Press => {
                if self.rect.contains(m.x, m.y) {
                    self.focused = true;
                    frame.request_focus(self.id);
                    self.cursor = (m.x.saturating_sub(self.rect.x + 1)) as usize;
                    self.cursor = self.cursor.min(self.value.chars().count());
                } else if self.focused {
                    self.focused = false;
                }
            }
            Event::Key(k) => {
                if !self.focused {
                    return;
                }
                match k.code {
                    KeyCode::Enter => {
                        self.submitted_flag = true;
                        if let Some(f) = self.on_submit.as_mut() {
                            f(&self.value);
                        }
                    }
                    KeyCode::Esc => self.focused = false,
                    KeyCode::Backspace => {
                        if self.cursor > 0 {
                            let idx = self.byte_at(self.cursor - 1);
                            let after = self.byte_at(self.cursor);
                            self.value.drain(idx..after);
                            self.cursor -= 1;
                            self.changed();
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor < self.value.chars().count() {
                            let idx = self.byte_at(self.cursor);
                            let after = self.byte_at(self.cursor + 1);
                            self.value.drain(idx..after);
                            self.changed();
                        }
                    }
                    KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor = (self.cursor + 1).min(self.value.chars().count())
                    }
                    KeyCode::Home => self.cursor = 0,
                    KeyCode::End => self.cursor = self.value.chars().count(),
                    KeyCode::Char(c) if !k.ctrl => {
                        if let Some(max) = self.max {
                            if self.value.chars().count() >= max {
                                return;
                            }
                        }
                        let idx = self.byte_at(self.cursor);
                        self.value.insert(idx, c);
                        self.cursor += 1;
                        self.changed();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn changed(&mut self) {
        if let Some(f) = self.on_change.as_mut() {
            f(&self.value);
        }
    }

    fn byte_at(&self, char_idx: usize) -> usize {
        self.value
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.value.len())
    }

    fn shown(&self) -> String {
        match self.mask {
            Some(m) => std::iter::repeat(m).take(self.value.chars().count()).collect(),
            None => self.value.clone(),
        }
    }

    /// Draw the field.
    pub fn draw(&mut self, frame: &mut Frame) {
        let is_focused = frame.register_focus();
        self.id = frame.focus_count() - 1;
        if is_focused {
            self.focused = true;
        }
        let st = if is_focused {
            self.focus_style
        } else {
            self.style
        };
        let mut c = Canvas::new(frame.buf);
        c.border(self.rect, border::thin(), st);

        let inner = self.rect.inner();
        let width = inner.w as usize;
        let shown = self.shown();
        let chars: Vec<char> = if shown.is_empty() && !is_focused {
            self.hint.chars().collect()
        } else {
            shown.chars().collect()
        };
        let hinting = shown.is_empty() && !is_focused;
        let hint_style = if hinting {
            st.patch(Style::default().dim(true).fg(Color::BrightBlack))
        } else {
            st
        };

        // view window so the cursor stays visible
        let cur = if hinting { 0 } else { self.cursor };
        let mut start = 0;
        if cur >= width {
            start = cur - width + 1;
        }
        let mut screen_x = inner.x as i32;
        for (i, ch) in chars.iter().enumerate().skip(start) {
            if screen_x >= inner.right() as i32 {
                break;
            }
            let cell_style = if is_focused && i == cur {
                st.patch(Style::default().reverse(true))
            } else {
                hint_style
            };
            c.put(screen_x, inner.y as i32, *ch, cell_style);
            screen_x += 1;
        }
        // trailing cursor block when focused and at end
        if is_focused && cur >= chars.len() && screen_x < inner.right() as i32 {
            c.put(screen_x, inner.y as i32, ' ', st.patch(Style::default().reverse(true)));
        }
    }
}
