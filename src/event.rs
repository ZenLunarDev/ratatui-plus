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

//! Input events: keyboard, mouse, resize, focus, tick. Includes a small
//! escape-sequence parser so the library needs no external crates.

/// A key press.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Key {
    pub fn new(code: KeyCode) -> Self {
        Key {
            code,
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    pub fn char(c: char) -> Self {
        Key::new(KeyCode::Char(c))
    }

    pub fn ctrl(mut self, v: bool) -> Self {
        self.ctrl = v;
        self
    }
    pub fn alt(mut self, v: bool) -> Self {
        self.alt = v;
        self
    }
    pub fn shift(mut self, v: bool) -> Self {
        self.shift = v;
        self
    }

    pub fn is_enter(&self) -> bool {
        self.code == KeyCode::Enter
    }
    pub fn is_esc(&self) -> bool {
        self.code == KeyCode::Esc
    }
    pub fn is_tab(&self) -> bool {
        self.code == KeyCode::Tab
    }
    pub fn is_backspace(&self) -> bool {
        self.code == KeyCode::Backspace
    }
    pub fn is_char(&self, c: char) -> bool {
        self.code == KeyCode::Char(c)
    }
    pub fn is_ctrl(&self, c: char) -> bool {
        self.ctrl && self.code == KeyCode::Char(c)
    }
    pub fn is(&self, c: KeyCode) -> bool {
        self.code == c
    }
}

/// What a key represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Esc,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,
    F(u8),
    Null,
}

/// Mouse button / wheel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
}

/// A mouse event (SGR 1006 encoded).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mouse {
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
    pub kind: MouseKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseKind {
    Press,
    Release,
    Motion,
}

/// Anything that can happen while the app is running.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Key(Key),
    Mouse(Mouse),
    Resize(u16, u16),
    Tick,
    Focus(bool),
    Quit,
}

/// Incremental parser turning raw bytes into [`Event`]s.
pub struct Parser {
    buf: Vec<u8>,
}

impl Parser {
    pub fn new() -> Self {
        Parser { buf: Vec::new() }
    }

    /// Feed raw bytes; produced events are appended to `out`.
    pub fn feed(&mut self, bytes: &[u8], out: &mut Vec<Event>) {
        self.buf.extend_from_slice(bytes);
        while let Some(n) = self.parse_one(out) {
            self.buf.drain(..n);
        }
    }

    /// Try to parse a single event from the front of `buf`.
    /// Returns `None` when more bytes are needed.
    fn parse_one(&mut self, out: &mut Vec<Event>) -> Option<usize> {
        let b = &self.buf;
        if b.is_empty() {
            return None;
        }
        let first = b[0];

        if first == 0x1b {
            // ESC
            if b.len() < 2 {
                return None;
            }
            match b[1] {
                b'[' => return self.parse_csi(out),
                b'O' => {
                    if b.len() >= 3 && (b'P'..=b'S').contains(&b[2]) {
                        let f = b[2] - b'P' + 1;
                        out.push(Event::Key(Key::new(KeyCode::F(f))));
                        return Some(3);
                    }
                    // focus out
                    out.push(Event::Focus(false));
                    return Some(2);
                }
                b'I' => {
                    out.push(Event::Focus(true));
                    return Some(2);
                }
                _ => {
                    // lone ESC -> Escape key
                    out.push(Event::Key(Key::new(KeyCode::Esc)));
                    return Some(1);
                }
            }
        }

        // control / printable
        let ev = match first {
            0x0d | 0x0a => Event::Key(Key::new(KeyCode::Enter)),
            0x7f | 0x08 => Event::Key(Key::new(KeyCode::Backspace)),
            0x09 => Event::Key(Key::new(KeyCode::Tab)),
            0x03 => Event::Key(Key::new(KeyCode::Char('c')).ctrl(true)),
            c if c < 0x20 => {
                let ch = (c + 0x60) as char;
                Event::Key(Key::new(KeyCode::Char(ch)).ctrl(true))
            }
            c => Event::Key(Key::new(KeyCode::Char(c as char))),
        };
        out.push(ev);
        Some(1)
    }

    fn parse_csi(&self, out: &mut Vec<Event>) -> Option<usize> {
        let b = &self.buf;
        if b.len() < 3 {
            return None;
        }
        let mut i = 2;
        while i < b.len() {
            let c = b[i];
            if (0x40..=0x7e).contains(&c) {
                break;
            }
            i += 1;
        }
        if i >= b.len() {
            return None; // wait for final byte
        }
        let final_byte = b[i];
        let body = &b[2..i];
        let used = i + 1;
        let is_mouse = body.first() == Some(&b'<');

        if is_mouse && (final_byte == b'M' || final_byte == b'm') {
            if let Some(m) = parse_mouse(body, final_byte) {
                out.push(Event::Mouse(m));
            }
            return Some(used);
        }

        let body_str = String::from_utf8_lossy(body);
        let code = match final_byte {
            b'A' => KeyCode::Up,
            b'B' => KeyCode::Down,
            b'C' => KeyCode::Right,
            b'D' => KeyCode::Left,
            b'H' => KeyCode::Home,
            b'F' => KeyCode::End,
            b'~' => map_tilde(&body_str),
            _ => KeyCode::Null,
        };
        if code != KeyCode::Null {
            out.push(Event::Key(Key::new(code)));
        }
        Some(used)
    }
}

fn map_tilde(params: &str) -> KeyCode {
    match params.trim() {
        "1" | "7" => KeyCode::Home,
        "2" => KeyCode::Insert,
        "3" => KeyCode::Delete,
        "4" | "8" => KeyCode::End,
        "5" => KeyCode::PageUp,
        "6" => KeyCode::PageDown,
        "11" => KeyCode::F(1),
        "12" => KeyCode::F(2),
        "13" => KeyCode::F(3),
        "14" => KeyCode::F(4),
        "15" => KeyCode::F(5),
        "17" => KeyCode::F(6),
        "18" => KeyCode::F(7),
        "19" => KeyCode::F(8),
        "20" => KeyCode::F(9),
        "21" => KeyCode::F(10),
        "23" => KeyCode::F(11),
        "24" => KeyCode::F(12),
        _ => KeyCode::Null,
    }
}

fn parse_mouse(body: &[u8], final_byte: u8) -> Option<Mouse> {
    let s = String::from_utf8_lossy(body).to_string();
    let s = s.trim_start_matches('<');
    let parts: Vec<&str> = s.split(';').collect();
    if parts.len() != 3 {
        return None;
    }
    let btn: u32 = parts[0].parse().ok()?;
    let x: u16 = parts[1].parse().ok()?;
    let y: u16 = parts[2].parse().ok()?;
    let x = x.saturating_sub(1);
    let y = y.saturating_sub(1);
    let kind = if final_byte == b'm' {
        MouseKind::Release
    } else if btn & 32 != 0 {
        MouseKind::Motion
    } else {
        MouseKind::Press
    };
    let button = if btn & 64 != 0 {
        if btn & 1 != 0 {
            MouseButton::WheelDown
        } else {
            MouseButton::WheelUp
        }
    } else {
        match btn & 3 {
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            _ => MouseButton::Left,
        }
    };
    Some(Mouse {
        x,
        y,
        button,
        kind,
    })
}
