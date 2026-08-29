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

//! Border styles — thin, double, rounded, thick, ASCII, dashed, and
//! curved (arc) variants. Each set is just the 8 glyphs that make a box.

/// The eight glyphs that draw a border around a rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BorderSet {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
    /// Glyph used for the rounded top corners (== top_left/right by default).
    pub top_left_curve: char,
    pub top_right_curve: char,
}

impl BorderSet {
    /// Build a set from the 6 core glyphs (corners default to top corners).
    pub fn new(
        tl: char,
        tr: char,
        bl: char,
        br: char,
        h: char,
        v: char,
    ) -> Self {
        BorderSet {
            top_left: tl,
            top_right: tr,
            bottom_left: bl,
            bottom_right: br,
            horizontal: h,
            vertical: v,
            top_left_curve: tl,
            top_right_curve: tr,
        }
    }

    /// A set where the top corners use curved arcs.
    pub fn with_curves(mut self, tlc: char, trc: char) -> Self {
        self.top_left_curve = tlc;
        self.top_right_curve = trc;
        self
    }
}

/// Single-line border.
pub fn thin() -> BorderSet {
    BorderSet::new('┌', '┐', '└', '┘', '─', '│')
}

/// Rounded (arc) border.
pub fn round() -> BorderSet {
    BorderSet::new('┌', '┐', '└', '┘', '─', '│').with_curves('╭', '╮')
}

/// Double-line border.
pub fn double() -> BorderSet {
    BorderSet::new('╔', '╗', '╚', '╝', '═', '║')
}

/// Thick border.
pub fn thick() -> BorderSet {
    BorderSet::new('┏', '┓', '┗', '┛', '━', '┃')
}

/// Block / full block border.
pub fn block() -> BorderSet {
    BorderSet::new('█', '█', '█', '█', '█', '█')
}

/// Dashed border (uses box-light dashes).
pub fn dashed() -> BorderSet {
    BorderSet::new('┌', '┐', '└', '┘', '╌', '╎')
}

/// Dotted border.
pub fn dotted() -> BorderSet {
    BorderSet::new('⋅', '⋅', '⋅', '⋅', '⋅', '⋅')
}

/// Full curved border — every corner is a quarter-circle arc.
pub fn curve() -> BorderSet {
    BorderSet::new('◜', '◝', '◟', '◞', '─', '│')
}

/// Classic ASCII border (safe on the oldest terminals).
pub fn ascii() -> BorderSet {
    BorderSet::new('+', '+', '+', '+', '-', '|')
}

/// A custom border from explicit glyphs.
pub fn custom(tl: char, tr: char, bl: char, br: char, h: char, v: char) -> BorderSet {
    BorderSet::new(tl, tr, bl, br, h, v)
}
