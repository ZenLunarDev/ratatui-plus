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

//! Colors — 4-bit, 8-bit (ansi256) and full 24-bit RGB, plus palettes
//! and gradients. Everything stays in the standard library.

use std::fmt::{self, Write};

/// A terminal color.
///
/// `Reset` restores the terminal default (which also reads as
/// "transparent" because it shows the terminal background).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Color {
    /// Terminal default.
    #[default]
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    /// Raw 8-bit palette index (0..=255).
    Ansi(u8),
    /// Full 24-bit true color.
    Rgb(u8, u8, u8),
}

/// How many colors the terminal can display. Used to auto-degrade
/// [`Color::Rgb`] so output still looks right on older terminals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorLevel {
    /// No color (e.g. `NO_COLOR` set).
    None,
    /// 4-bit / 16 colors.
    Basic,
    /// 8-bit / 256 colors.
    Ansi256,
    /// 24-bit true color.
    Rgb,
}

impl ColorLevel {
    pub fn supports_truecolor(self) -> bool {
        self == ColorLevel::Rgb
    }
    pub fn supports_256(self) -> bool {
        matches!(self, ColorLevel::Ansi256 | ColorLevel::Rgb)
    }
}

impl Color {
    /// Downgrade this color to what `level` can display (auto-degradation).
    pub fn degrade(self, level: ColorLevel) -> Color {
        match level {
            ColorLevel::Rgb => self,
            ColorLevel::None => Color::Reset,
            ColorLevel::Ansi256 => match self {
                Color::Rgb(r, g, b) => Color::Ansi(nearest_ansi(r, g, b)),
                Color::Ansi(c) => Color::Ansi(c),
                other => other,
            },
            ColorLevel::Basic => match self {
                Color::Rgb(r, g, b) => nearest_basic(r, g, b),
                Color::Ansi(c) => ansi_to_basic(c),
                other => other,
            },
        }
    }
    /// Build a true-color from components.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::Rgb(r, g, b)
    }

    /// Parse `#rrggbb` or `#rgb`.
    pub fn hex(s: &str) -> Self {
        let s = s.trim_start_matches('#');
        match s.len() {
            3 => {
                let r = u8::from_str_radix(&s[0..1], 16).map(|v| v * 17).unwrap_or(0);
                let g = u8::from_str_radix(&s[1..2], 16).map(|v| v * 17).unwrap_or(0);
                let b = u8::from_str_radix(&s[2..3], 16).map(|v| v * 17).unwrap_or(0);
                Color::Rgb(r, g, b)
            }
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
                Color::Rgb(r, g, b)
            }
            _ => Color::Reset,
        }
    }

    /// Linear interpolation between two colors (RGB space).
    pub fn mix(self, other: Color, t: f32) -> Color {
        let a = self.to_rgb();
        let b = other.to_rgb();
        let t = t.clamp(0.0, 1.0);
        let r = (a.0 as f32 + (b.0 as f32 - a.0 as f32) * t).round() as u8;
        let g = (a.1 as f32 + (b.1 as f32 - a.1 as f32) * t).round() as u8;
        let bl = (a.2 as f32 + (b.2 as f32 - a.2 as f32) * t).round() as u8;
        Color::Rgb(r, g, bl)
    }

    /// Build an `n`-step gradient from `self` to `end` (inclusive).
    pub fn gradient(self, end: Color, n: usize) -> Vec<Color> {
        if n == 0 {
            return Vec::new();
        }
        (0..n)
            .map(|i| self.mix(end, i as f32 / (n as f32 - 1.0).max(1.0)))
            .collect()
    }

    /// Convert any color to its 24-bit RGB triple.
    pub fn to_rgb(self) -> (u8, u8, u8) {
        match self {
            Color::Reset => (0, 0, 0),
            Color::Black => (0, 0, 0),
            Color::Red => (204, 0, 0),
            Color::Green => (0, 175, 0),
            Color::Yellow => (204, 204, 0),
            Color::Blue => (0, 0, 204),
            Color::Magenta => (204, 0, 204),
            Color::Cyan => (0, 175, 175),
            Color::White => (187, 187, 187),
            Color::BrightBlack => (85, 85, 85),
            Color::BrightRed => (255, 85, 85),
            Color::BrightGreen => (0, 255, 0),
            Color::BrightYellow => (255, 255, 85),
            Color::BrightBlue => (85, 85, 255),
            Color::BrightMagenta => (255, 85, 255),
            Color::BrightCyan => (85, 255, 255),
            Color::BrightWhite => (255, 255, 255),
            Color::Ansi(c) => ANSI_256[(c as usize).min(255)],
            Color::Rgb(r, g, b) => (r, g, b),
        }
    }

    /// SGR foreground sequence (`38;...`).
    pub(crate) fn fg_code(&self, out: &mut String) {
        match self {
            Color::Reset => out.push_str("39"),
            Color::Black => out.push_str("30"),
            Color::Red => out.push_str("31"),
            Color::Green => out.push_str("32"),
            Color::Yellow => out.push_str("33"),
            Color::Blue => out.push_str("34"),
            Color::Magenta => out.push_str("35"),
            Color::Cyan => out.push_str("36"),
            Color::White => out.push_str("37"),
            Color::BrightBlack => out.push_str("90"),
            Color::BrightRed => out.push_str("91"),
            Color::BrightGreen => out.push_str("92"),
            Color::BrightYellow => out.push_str("93"),
            Color::BrightBlue => out.push_str("94"),
            Color::BrightMagenta => out.push_str("95"),
            Color::BrightCyan => out.push_str("96"),
            Color::BrightWhite => out.push_str("97"),
            Color::Ansi(c) => {
                let _ = write!(out, "38;5;{c}");
            }
            Color::Rgb(r, g, b) => {
                let _ = write!(out, "38;2;{r};{g};{b}");
            }
        }
    }

    /// SGR background sequence (`48;...`).
    pub(crate) fn bg_code(&self, out: &mut String) {
        match self {
            Color::Reset => out.push_str("49"),
            Color::Black => out.push_str("40"),
            Color::Red => out.push_str("41"),
            Color::Green => out.push_str("42"),
            Color::Yellow => out.push_str("43"),
            Color::Blue => out.push_str("44"),
            Color::Magenta => out.push_str("45"),
            Color::Cyan => out.push_str("46"),
            Color::White => out.push_str("47"),
            Color::BrightBlack => out.push_str("100"),
            Color::BrightRed => out.push_str("101"),
            Color::BrightGreen => out.push_str("102"),
            Color::BrightYellow => out.push_str("103"),
            Color::BrightBlue => out.push_str("104"),
            Color::BrightMagenta => out.push_str("105"),
            Color::BrightCyan => out.push_str("106"),
            Color::BrightWhite => out.push_str("107"),
            Color::Ansi(c) => {
                let _ = write!(out, "48;5;{c}");
            }
            Color::Rgb(r, g, b) => {
                let _ = write!(out, "48;2;{r};{g};{b}");
            }
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Reset => write!(f, "default"),
            Color::Rgb(r, g, b) => write!(f, "#{r:02x}{g:02x}{b:02x}"),
            Color::Ansi(c) => write!(f, "ansi{c}"),
            other => write!(f, "{other:?}"),
        }
    }
}

/// The xterm 256-color cube + grayscale ramp as RGB triples.
const ANSI_256: [(u8, u8, u8); 256] = {
    let mut t = [(0u8, 0u8, 0u8); 256];
    let mut i = 0usize;
    // 0..=15 standard + bright
    let std = [
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];
    while i < 16 {
        t[i] = std[i];
        i += 1;
    }
    // 16..=231 color cube
    let vals = [0u8, 95, 135, 175, 215, 255];
    let mut r = 0;
    while r < 6 {
        let mut g = 0;
        while g < 6 {
            let mut b = 0;
            while b < 6 {
                t[i] = (vals[r], vals[g], vals[b]);
                i += 1;
                b += 1;
            }
            g += 1;
        }
        r += 1;
    }
    // 232..=255 grayscale
    let mut v = 8u8;
    while i < 256 {
        t[i] = (v, v, v);
        v = v.saturating_add(10);
        i += 1;
    }
    t
};

/// The 16 standard colors as RGB triples (for nearest-color matching).
const BASIC_16: [(Color, (u8, u8, u8)); 16] = [
    (Color::Black, (0, 0, 0)),
    (Color::Red, (204, 0, 0)),
    (Color::Green, (0, 175, 0)),
    (Color::Yellow, (204, 204, 0)),
    (Color::Blue, (0, 0, 204)),
    (Color::Magenta, (204, 0, 204)),
    (Color::Cyan, (0, 175, 175)),
    (Color::White, (187, 187, 187)),
    (Color::BrightBlack, (85, 85, 85)),
    (Color::BrightRed, (255, 85, 85)),
    (Color::BrightGreen, (0, 255, 0)),
    (Color::BrightYellow, (255, 255, 85)),
    (Color::BrightBlue, (85, 85, 255)),
    (Color::BrightMagenta, (255, 85, 255)),
    (Color::BrightCyan, (85, 255, 255)),
    (Color::BrightWhite, (255, 255, 255)),
];

fn dist2(a: (u8, u8, u8), b: (u8, u8, u8)) -> u32 {
    let dr = a.0 as i32 - b.0 as i32;
    let dg = a.1 as i32 - b.1 as i32;
    let db = a.2 as i32 - b.2 as i32;
    (dr * dr + dg * dg + db * db) as u32
}

fn nearest_ansi(r: u8, g: u8, b: u8) -> u8 {
    let mut best = 0u8;
    let mut best_d = u32::MAX;
    for (i, c) in ANSI_256.iter().enumerate() {
        let d = dist2((r, g, b), *c);
        if d < best_d {
            best_d = d;
            best = i as u8;
        }
    }
    best
}

fn nearest_basic(r: u8, g: u8, b: u8) -> Color {
    let mut best = Color::White;
    let mut best_d = u32::MAX;
    for (col, c) in BASIC_16.iter() {
        let d = dist2((r, g, b), *c);
        if d < best_d {
            best_d = d;
            best = *col;
        }
    }
    best
}

fn ansi_to_basic(c: u8) -> Color {
    BASIC_16[(c as usize).min(15)].0
}

/// Handy re-exports of the basic palette so callers can do `use ratatui_plus::color::*;`.
pub mod palette {
    use super::Color;
    pub const BLACK: Color = Color::Black;
    pub const WHITE: Color = Color::White;
    pub const RED: Color = Color::Red;
    pub const GREEN: Color = Color::Green;
    pub const BLUE: Color = Color::Blue;
    pub const YELLOW: Color = Color::Yellow;
    pub const CYAN: Color = Color::Cyan;
    pub const MAGENTA: Color = Color::Magenta;
    pub const ORANGE: Color = Color::Rgb(255, 140, 0);
    pub const PINK: Color = Color::Rgb(255, 105, 180);
    pub const PURPLE: Color = Color::Rgb(147, 112, 219);
    pub const TEAL: Color = Color::Rgb(0, 128, 128);
    pub const GOLD: Color = Color::Rgb(255, 215, 0);
    pub const SLATE: Color = Color::Rgb(112, 128, 144);
    pub const DARK: Color = Color::Rgb(24, 24, 32);
}
