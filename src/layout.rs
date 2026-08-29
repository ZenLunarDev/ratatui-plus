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

/// Center a `w`×`h` box inside this rect (clamped to fit).
impl Rect {
    pub fn centered(self, w: u16, h: u16) -> Rect {
        let w = w.min(self.w);
        let h = h.min(self.h);
        let x = self.x + (self.w - w) / 2;
        let y = self.y + (self.h - h) / 2;
        Rect::new(x, y, w, h)
    }

    /// Alias for [`Rect::centered`].
    pub fn fit(self, w: u16, h: u16) -> Rect {
        self.centered(w, h)
    }
}

/// A layout direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

/// A sizing constraint used by [`Layout`] to auto-size regions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Constraint {
    /// Fixed number of cells.
    Length(u16),
    /// A percentage (0..=100) of the available space.
    Percentage(u16),
    /// A ratio `num / den` of the available space.
    Ratio(u32, u32),
    /// At least this many cells, but may grow.
    Min(u16),
    /// At most this many cells.
    Max(u16),
    /// Take all remaining space (shares equally with other `Fill`/`Auto`).
    Fill,
    /// Auto: minimal size, grows to fill remaining space.
    Auto,
}

/// A responsive layout that splits a rect into regions automatically.
///
/// ```ignore
/// let chunks = Layout::vertical([
///     Constraint::Length(3),        // header
///     Constraint::Fill,             // body takes the rest
///     Constraint::Percentage(15),   // footer
/// ]).split(area);
/// ```
#[derive(Clone, Debug)]
pub struct Layout {
    direction: Direction,
    constraints: Vec<Constraint>,
    margin: u16,
    spacing: u16,
}

impl Layout {
    /// A vertical (top-to-bottom) layout.
    pub fn vertical<C: Into<Constraint>>(c: impl IntoIterator<Item = C>) -> Self {
        Layout::new(Direction::Vertical, c)
    }

    /// A horizontal (left-to-right) layout.
    pub fn horizontal<C: Into<Constraint>>(c: impl IntoIterator<Item = C>) -> Self {
        Layout::new(Direction::Horizontal, c)
    }

    /// Build a layout with an explicit direction.
    pub fn new<C: Into<Constraint>>(direction: Direction, c: impl IntoIterator<Item = C>) -> Self {
        Layout {
            direction,
            constraints: c.into_iter().map(|x| x.into()).collect(),
            margin: 0,
            spacing: 0,
        }
    }

    /// Add a single constraint (builder style).
    pub fn constraint(mut self, c: Constraint) -> Self {
        self.constraints.push(c);
        self
    }

    /// Outer margin (cells on every side).
    pub fn margin(mut self, n: u16) -> Self {
        self.margin = n;
        self
    }

    /// Gap between regions (cells).
    pub fn spacing(mut self, n: u16) -> Self {
        self.spacing = n;
        self
    }

    /// Compute the resulting rects for `area`.
    pub fn split(self, area: Rect) -> Vec<Rect> {
        let n = self.constraints.len();
        if n == 0 {
            return Vec::new();
        }
        let inner = Rect::new(
            area.x + self.margin,
            area.y + self.margin,
            area.w.saturating_sub(self.margin * 2),
            area.h.saturating_sub(self.margin * 2),
        );
        let total = if self.direction == Direction::Horizontal {
            inner.w
        } else {
            inner.h
        };
        let gap = self.spacing * (n as u16 - 1);
        let usable = total.saturating_sub(gap);

        let mut sizes = vec![0u16; n];
        let mut remaining = usable as i64;

        // Pass 1: resolve fixed/limited constraints and Min baseline.
        for i in 0..n {
            let want = match self.constraints[i] {
                Constraint::Length(v) => v as i64,
                Constraint::Percentage(p) => (usable as i64 * p as i64) / 100,
                Constraint::Ratio(a, b) => {
                    if b == 0 {
                        0
                    } else {
                        (usable as i64 * a as i64) / b as i64
                    }
                }
                Constraint::Max(v) => (v as i64).min(remaining),
                Constraint::Min(v) => (v as i64).min(remaining),
                Constraint::Fill | Constraint::Auto => 0,
            };
            let want = want.max(0).min(remaining);
            sizes[i] = want as u16;
            match self.constraints[i] {
                Constraint::Length(_)
                | Constraint::Percentage(_)
                | Constraint::Ratio(..)
                | Constraint::Max(_)
                | Constraint::Min(_) => {
                    remaining -= want;
                }
                _ => {}
            }
        }

        // Pass 2: distribute any leftover space among growing constraints.
        let grows: Vec<usize> = (0..n)
            .filter(|&i| matches!(self.constraints[i], Constraint::Fill | Constraint::Auto | Constraint::Min(_)))
            .collect();
        let mut guard = 0;
        while remaining > 0 && !grows.is_empty() && guard < n as i32 + 4 {
            guard += 1;
            let share = (remaining / grows.len() as i64).max(1);
            for &i in &grows {
                let add = share.min(remaining);
                if add <= 0 {
                    continue;
                }
                sizes[i] += add as u16;
                remaining -= add;
            }
        }

        // Position the regions.
        let mut rects = Vec::with_capacity(n);
        let mut pos = if self.direction == Direction::Horizontal {
            inner.x
        } else {
            inner.y
        };
        for i in 0..n {
            let s = sizes[i];
            let r = if self.direction == Direction::Horizontal {
                Rect::new(pos, inner.y, s, inner.h)
            } else {
                Rect::new(inner.x, pos, inner.w, s)
            };
            rects.push(r);
            pos += s + self.spacing;
        }
        rects
    }
}

impl From<u16> for Constraint {
    fn from(v: u16) -> Self {
        Constraint::Length(v)
    }
}

impl From<(u32, u32)> for Constraint {
    fn from((a, b): (u32, u32)) -> Self {
        Constraint::Ratio(a, b)
    }
}
