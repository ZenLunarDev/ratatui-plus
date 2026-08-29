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

//! Low-level drawing primitives operating directly on a [`Buffer`].

use crate::buffer::Buffer;
use crate::border::BorderSet;
use crate::layout::{Align, Point, Rect};
use crate::style::Style;

/// Draw into a [`Buffer`] with pixel-less, cell-based primitives.
pub struct Canvas<'a> {
    buf: &'a mut Buffer,
    clip: Rect,
}

impl<'a> Canvas<'a> {
    pub fn new(buf: &'a mut Buffer) -> Self {
        let clip = buf.area;
        Canvas { buf, clip }
    }

    /// Restrict drawing to a clipping rect.
    pub fn clip(mut self, r: Rect) -> Self {
        self.clip = self.clip.intersect(r);
        self
    }

    fn put(&mut self, x: i32, y: i32, ch: char, s: Style) {
        if x >= 0 && y >= 0 {
            let (ux, uy) = (x as u16, y as u16);
            if self.clip.contains(ux, uy) {
                self.buf.put(ux, uy, ch, s);
            }
        }
    }

    /// A single dot.
    pub fn dot(&mut self, x: i32, y: i32, ch: char, s: Style) {
        self.put(x, y, ch, s);
    }

    /// A horizontal line.
    pub fn hline(&mut self, x: i32, y: i32, len: i32, ch: char, s: Style) {
        let (a, b) = if len < 0 { (x + len, x) } else { (x, x + len) };
        for cx in a..=b {
            self.put(cx, y, ch, s);
        }
    }

    /// A vertical line.
    pub fn vline(&mut self, x: i32, y: i32, len: i32, ch: char, s: Style) {
        let (a, b) = if len < 0 { (y + len, y) } else { (y, y + len) };
        for cy in a..=b {
            self.put(x, cy, ch, s);
        }
    }

    /// Bresenham line between two points, choosing the best glyph for
    /// horizontal / vertical / diagonal runs.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, ch: char, s: Style) {
        if y0 == y1 {
            self.hline(x0, y0, x1 - x0, ch, s);
            return;
        }
        if x0 == x1 {
            self.vline(x0, y0, y1 - y0, ch, s);
            return;
        }
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (x0, y0);
        loop {
            self.put(x, y, ch, s);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Stroke a rectangle outline.
    pub fn rect(&mut self, r: Rect, ch: char, s: Style) {
        let x0 = r.x as i32;
        let y0 = r.y as i32;
        let x1 = (r.right() as i32).saturating_sub(1);
        let y1 = (r.bottom() as i32).saturating_sub(1);
        if r.w == 0 || r.h == 0 {
            return;
        }
        self.hline(x0, y0, r.w as i32 - 1, ch, s);
        self.hline(x0, y1, r.w as i32 - 1, ch, s);
        self.vline(x0, y0, r.h as i32 - 1, ch, s);
        self.vline(x1, y0, r.h as i32 - 1, ch, s);
        self.put(x0, y0, ch, s);
        self.put(x1, y0, ch, s);
        self.put(x0, y1, ch, s);
        self.put(x1, y1, ch, s);
    }

    /// Filled rectangle.
    pub fn fill_rect(&mut self, r: Rect, ch: char, s: Style) {
        for y in r.y..r.bottom() {
            for x in r.x..r.right() {
                self.put(x as i32, y as i32, ch, s);
            }
        }
    }

    /// A rectangle drawn with a [`BorderSet`].
    pub fn border(&mut self, r: Rect, b: BorderSet, s: Style) {
        if r.w == 0 || r.h == 0 {
            return;
        }
        let x0 = r.x as i32;
        let y0 = r.y as i32;
        let x1 = (r.right() as i32).saturating_sub(1);
        let y1 = (r.bottom() as i32).saturating_sub(1);
        self.hline(x0 + 1, y0, r.w as i32 - 2, b.horizontal, s);
        self.hline(x0 + 1, y1, r.w as i32 - 2, b.horizontal, s);
        self.vline(x0, y0 + 1, r.h as i32 - 2, b.vertical, s);
        self.vline(x1, y0 + 1, r.h as i32 - 2, b.vertical, s);
        self.put(x0, y0, b.top_left_curve, s);
        self.put(x1, y0, b.top_right_curve, s);
        self.put(x0, y1, b.bottom_left, s);
        self.put(x1, y1, b.bottom_right, s);
    }

    /// A circle (midpoint algorithm) centered at `(cx, cy)` radius `rad`.
    pub fn circle(&mut self, cx: i32, cy: i32, rad: i32, ch: char, s: Style) {
        let mut x = rad;
        let mut y = 0i32;
        let mut err = 1 - rad;
        while x >= y {
            for &(px, py) in &[
                (cx + x, cy + y),
                (cx + y, cy + x),
                (cx - y, cy + x),
                (cx - x, cy + y),
                (cx - x, cy - y),
                (cx - y, cy - x),
                (cx + y, cy - x),
                (cx + x, cy - y),
            ] {
                self.put(px, py, ch, s);
            }
            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    /// A filled circle.
    pub fn disc(&mut self, cx: i32, cy: i32, rad: i32, ch: char, s: Style) {
        for y in -rad..=rad {
            for x in -rad..=rad {
                if x * x + y * y <= rad * rad {
                    self.put(cx + x, cy + y, ch, s);
                }
            }
        }
    }

    /// A quadratic Bézier curve (used for "curved" decorations).
    pub fn curve(&mut self, p0: Point, p1: Point, p2: Point, ch: char, s: Style) {
        let steps = 64u32;
        let (x0, y0) = (p0.x as f32, p0.y as f32);
        let (x1, y1) = (p1.x as f32, p1.y as f32);
        let (x2, y2) = (p2.x as f32, p2.y as f32);
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let mt = 1.0 - t;
            let x = mt * mt * x0 + 2.0 * mt * t * x1 + t * t * x2;
            let y = mt * mt * y0 + 2.0 * mt * t * y1 + t * t * y2;
            self.put(x.round() as i32, y.round() as i32, ch, s);
        }
    }

    /// Plot a polyline from a list of points.
    pub fn polyline(&mut self, pts: &[Point], ch: char, s: Style) {
        for w in pts.windows(2) {
            self.line(w[0].x as i32, w[0].y as i32, w[1].x as i32, w[1].y as i32, ch, s);
        }
    }

    /// Render text at `(x, y)` with alignment inside `width`.
    pub fn text(&mut self, x: i32, y: i32, s: &str, align: Align, width: u16, st: Style) {
        let len = s.chars().count() as u16;
        let off = align.h_offset(len, width);
        self.buf.write_str((x + off as i32) as u16, y as u16, s, st);
    }
}
