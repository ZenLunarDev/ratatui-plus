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

//! Geometry: [`Rect`], [`Point`], [`Align`] and layout helpers.

/// A 2D point (column `x`, row `y`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub const fn new(x: u16, y: u16) -> Self {
        Point { x, y }
    }
}

/// A rectangle defined by its top-left origin and size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    /// Build a rectangle from origin + size.
    pub const fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Rect { x, y, w, h }
    }

    /// Set the origin.
    pub fn xy(mut self, x: u16, y: u16) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set the size.
    pub fn wh(mut self, w: u16, h: u16) -> Self {
        self.w = w;
        self.h = h;
        self
    }

    /// Area in cells.
    pub const fn area(self) -> u32 {
        self.w as u32 * self.h as u32
    }

    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.w)
    }

    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.h)
    }

    pub const fn contains(self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Shrink the rect by `n` cells on every side. Clamped to zero.
    pub fn padding(self, n: u16) -> Rect {
        Rect {
            x: self.x.saturating_add(n),
            y: self.y.saturating_add(n),
            w: self.w.saturating_sub(n * 2),
            h: self.h.saturating_sub(n * 2),
        }
    }

    /// Inset by different horizontal / vertical amounts.
    pub fn inset(self, h: u16, v: u16) -> Rect {
        Rect {
            x: self.x.saturating_add(h),
            y: self.y.saturating_add(v),
            w: self.w.saturating_sub(h * 2),
            h: self.h.saturating_sub(v * 2),
        }
    }

    /// The inner rect (1 cell border) — handy for panels.
    pub fn inner(self) -> Rect {
        self.padding(1)
    }

    /// Center point of the rect.
    pub fn center(self) -> Point {
        Point::new(self.x + self.w / 2, self.y + self.h / 2)
    }

    /// Intersection with another rect (may be empty).
    pub fn intersect(self, o: Rect) -> Rect {
        let x = self.x.max(o.x);
        let y = self.y.max(o.y);
        let r = self.right().min(o.right());
        let b = self.bottom().min(o.bottom());
        Rect {
            x,
            y,
            w: r.saturating_sub(x),
            h: b.saturating_sub(y),
        }
    }

    /// Clip a rect to `bound` and return whether anything remains.
    pub fn clip_to(self, bound: Rect) -> Rect {
        self.intersect(bound)
    }

    /// Split vertically into `n` equal columns with optional gap.
    pub fn split_v(self, n: u16, gap: u16) -> Vec<Rect> {
        if n == 0 {
            return Vec::new();
        }
        let total_gap = gap * (n.saturating_sub(1));
        let w = (self.w as i32 - total_gap as i32).max(0) as u16 / n;
        let mut out = Vec::with_capacity(n as usize);
        let mut x = self.x;
        for _ in 0..n {
            out.push(Rect::new(x, self.y, w, self.h));
            x = x.saturating_add(w).saturating_add(gap);
        }
        out
    }

    /// Split horizontally into `n` equal rows with optional gap.
    pub fn split_h(self, n: u16, gap: u16) -> Vec<Rect> {
        if n == 0 {
            return Vec::new();
        }
        let total_gap = gap * (n.saturating_sub(1));
        let h = (self.h as i32 - total_gap as i32).max(0) as u16 / n;
        let mut out = Vec::with_capacity(n as usize);
        let mut y = self.y;
        for _ in 0..n {
            out.push(Rect::new(self.x, y, self.w, h));
            y = y.saturating_add(h).saturating_add(gap);
        }
        out
    }
}

/// Horizontal + vertical alignment inside a box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
}

impl Align {
    /// Horizontal placement of `len` cells within `width`.
    pub fn h_offset(self, len: u16, width: u16) -> u16 {
        match self {
            Align::Center | Align::Middle => (width.saturating_sub(len)) / 2,
            Align::Right | Align::Bottom => width.saturating_sub(len),
            _ => 0,
        }
    }

    /// Vertical placement of `height` cells within `rows`.
    pub fn v_offset(self, height: u16, rows: u16) -> u16 {
        match self {
            Align::Middle | Align::Center => (rows.saturating_sub(height)) / 2,
            Align::Bottom => rows.saturating_sub(height),
            _ => 0,
        }
    }
}
