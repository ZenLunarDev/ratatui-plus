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

//! `Style` — foreground/background, text attributes and transparency.
//!
//! Transparency is expressed two ways: with [`Color::Reset`] (show the
//! terminal background) or with [`Style::alpha`] which emits an RGBA
//! escape supported by Kitty-style terminals and otherwise degrades to
//! the closest solid color.

use crate::color::{Color, ColorLevel};
use std::fmt::Write;

/// A cell's visual style.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub double_underline: bool,
    pub strikethrough: bool,
    pub blink: bool,
    pub fast_blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    /// 0.0 = fully transparent (alpha escape), 1.0 = opaque.
    pub alpha: f32,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            fg: Color::Reset,
            bg: Color::Reset,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            double_underline: false,
            strikethrough: false,
            blink: false,
            fast_blink: false,
            reverse: false,
            hidden: false,
            alpha: 1.0,
        }
    }
}

impl Style {
    /// Start from an empty (reset) style.
    pub fn new() -> Self {
        Style::default()
    }

    pub fn fg(mut self, c: Color) -> Self {
        self.fg = c;
        self
    }

    pub fn bg(mut self, c: Color) -> Self {
        self.bg = c;
        self
    }

    pub fn color(mut self, fg: Color, bg: Color) -> Self {
        self.fg = fg;
        self.bg = bg;
        self
    }

    pub fn bold(mut self, v: bool) -> Self {
        self.bold = v;
        self
    }
    pub fn dim(mut self, v: bool) -> Self {
        self.dim = v;
        self
    }
    pub fn italic(mut self, v: bool) -> Self {
        self.italic = v;
        self
    }
    pub fn underline(mut self, v: bool) -> Self {
        self.underline = v;
        self
    }
    pub fn dunderline(mut self, v: bool) -> Self {
        self.double_underline = v;
        self
    }
    pub fn strike(mut self, v: bool) -> Self {
        self.strikethrough = v;
        self
    }
    pub fn blink(mut self, v: bool) -> Self {
        self.blink = v;
        self
    }
    pub fn fast_blink(mut self, v: bool) -> Self {
        self.fast_blink = v;
        self
    }
    pub fn reverse(mut self, v: bool) -> Self {
        self.reverse = v;
        self
    }
    pub fn hidden(mut self, v: bool) -> Self {
        self.hidden = v;
        self
    }

    /// Set background transparency, `0.0..=1.0`.
    pub fn alpha(mut self, a: f32) -> Self {
        self.alpha = a.clamp(0.0, 1.0);
        self
    }

    /// Shorthand: fully transparent background.
    pub fn transparent(mut self) -> Self {
        self.bg = Color::Reset;
        self.alpha = 0.0;
        self
    }

    /// Merge another style on top (non-default fields win).
    pub fn patch(mut self, o: Style) -> Self {
        if o.fg != Color::Reset {
            self.fg = o.fg;
        }
        if o.bg != Color::Reset {
            self.bg = o.bg;
        }
        self.bold |= o.bold;
        self.dim |= o.dim;
        self.italic |= o.italic;
        self.underline |= o.underline;
        self.double_underline |= o.double_underline;
        self.strikethrough |= o.strikethrough;
        self.blink |= o.blink;
        self.fast_blink |= o.fast_blink;
        self.reverse |= o.reverse;
        self.hidden |= o.hidden;
        if o.alpha < 1.0 {
            self.alpha = o.alpha;
        }
        self
    }

    /// Build the opening SGR sequence for this style, auto-degraded to
    /// the terminal's [`ColorLevel`].
    pub(crate) fn open(&self, level: ColorLevel) -> String {
        let mut p: Vec<String> = Vec::new();
        if self.bold {
            p.push("1".into());
        }
        if self.dim {
            p.push("2".into());
        }
        if self.italic {
            p.push("3".into());
        }
        if self.underline {
            p.push("4".into());
        }
        if self.double_underline {
            p.push("21".into());
        }
        if self.blink {
            p.push("5".into());
        }
        if self.fast_blink {
            p.push("6".into());
        }
        if self.reverse {
            p.push("7".into());
        }
        if self.hidden {
            p.push("8".into());
        }
        if self.strikethrough {
            p.push("9".into());
        }

        let fg = self.fg.degrade(level);
        let bg = self.bg.degrade(level);

        let mut s = String::new();
        // foreground
        if self.alpha < 1.0 {
            if let Color::Rgb(r, g, b) = fg {
                let a = (self.alpha * 255.0).round() as u8;
                let _ = write!(s, "38:2::{r}:{g}:{b}:{a};");
            } else {
                fg.fg_code(&mut s);
                s.push(';');
            }
        } else if fg != Color::Reset {
            fg.fg_code(&mut s);
            s.push(';');
        }
        // background
        if bg != Color::Reset {
            if self.alpha < 1.0 {
                if let Color::Rgb(r, g, b) = bg {
                    let a = (self.alpha * 255.0).round() as u8;
                    let _ = write!(s, "48:2::{r}:{g}:{b}:{a};");
                } else {
                    bg.bg_code(&mut s);
                    s.push(';');
                }
            } else {
                bg.bg_code(&mut s);
                s.push(';');
            }
        }
        for part in &p {
            s.push_str(part);
            s.push(';');
        }
        if s.ends_with(';') {
            s.pop();
        }
        format!("\x1b[{s}m")
    }

    /// Reset sequence.
    pub(crate) const RESET: &'static str = "\x1b[0m";
}

/// Convenience: a bold foreground style.
pub fn fg(c: Color) -> Style {
    Style::new().fg(c)
}
