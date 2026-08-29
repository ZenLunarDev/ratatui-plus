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

//! The cell grid that widgets draw into, and a minimal diff renderer.

use crate::layout::Rect;
use crate::style::{Style, Style::RESET};
use std::cmp::min;

/// A single terminal cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
}

impl Cell {
    pub fn new(ch: char, style: Style) -> Self {
        Cell { ch, style }
    }

    pub fn blank() -> Self {
        Cell {
            ch: ' ',
            style: Style::default(),
        }
    }
}

/// A grid of cells covering a rectangular area.
#[derive(Clone, Debug)]
pub struct Buffer {
    pub area: Rect,
    cells: Vec<Cell>,
}

impl Buffer {
    /// Create a blank buffer.
    pub fn empty(area: Rect) -> Self {
        let n = (area.w as usize).saturating_mul(area.h as usize);
        Buffer {
            area,
            cells: vec![Cell::blank(); n],
        }
    }

    pub fn resize(&mut self, area: Rect) {
        if self.area == area {
            return;
        }
        let mut nb = Buffer::empty(area);
        // copy overlapping region
        let w = min(self.area.w, area.w) as usize;
        let h = min(self.area.h, area.h) as usize;
        for y in 0..h {
            for x in 0..w {
                nb.cells[y * area.w as usize + x] = self.cells[y * self.area.w as usize + x];
            }
        }
        *self = nb;
    }

    /// Reset every cell to blank.
    pub fn clear(&mut self) {
        for c in self.cells.iter_mut() {
            *c = Cell::blank();
        }
    }

    fn idx(&self, x: u16, y: u16) -> Option<usize> {
        if x < self.area.w && y < self.area.h {
            Some(y as usize * self.area.w as usize + x as usize)
        } else {
            None
        }
    }

    /// Get a cell (out-of-bounds returns a blank copy without storing).
    pub fn get(&self, x: u16, y: u16) -> Cell {
        self.idx(x, y)
            .map(|i| self.cells[i])
            .unwrap_or_else(Cell::blank)
    }

    /// Set a cell.
    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(i) = self.idx(x, y) {
            self.cells[i] = cell;
        }
    }

    /// Set a character with a style at `(x, y)`.
    pub fn put(&mut self, x: u16, y: u16, ch: char, style: Style) {
        self.set(x, y, Cell::new(ch, style));
    }

    /// Write a string at `(x, y)`, advancing horizontally, clipping to the buffer.
    pub fn write_str(&mut self, x: u16, y: u16, s: &str, style: Style) {
        let mut cx = x;
        for ch in s.chars() {
            if cx >= self.area.w {
                break;
            }
            self.put(cx, y, ch, style);
            cx += 1;
        }
    }

    /// Fill the whole buffer with one cell.
    pub fn fill(&mut self, ch: char, style: Style) {
        for c in self.cells.iter_mut() {
            *c = Cell::new(ch, style);
        }
    }

    /// Overlay `other` onto this buffer at its origin (used by widgets/clipping).
    pub fn merge(&mut self, other: &Buffer, ox: u16, oy: u16) {
        for y in 0..other.area.h {
            for x in 0..other.area.w {
                let c = other.get(x, y);
                let dx = ox + x;
                let dy = oy + y;
                if dx < self.area.w && dy < self.area.h {
                    self.set(dx, dy, c);
                }
            }
        }
    }

    /// Produce an ANSI string that updates `prev` into `self`.
    ///
    /// When `prev` is `None`, the whole buffer is emitted. This diff keeps
    /// real-time rendering smooth (no full-screen clears / flicker).
    pub fn diff(&self, prev: Option<&Buffer>) -> String {
        let mut out = String::new();
        let w = self.area.w;
        let h = self.area.h;
        match prev {
            None => {
                for y in 0..h {
                    for x in 0..w {
                        let i = y as usize * w as usize + x as usize;
                        let cell = &self.cells[i];
                        out.push_str(&format!("\x1b[{};{}H", y + 1, x + 1));
                        out.push_str(&cell.style.open());
                        out.push(cell.ch);
                        out.push_str(RESET);
                    }
                }
            }
            Some(prev) => {
                let pw = prev.area.w;
                let ph = prev.area.h;
                for y in 0..h {
                    let mut x = 0u16;
                    while x < w {
                        let i = y as usize * w as usize + x as usize;
                        let cell = &self.cells[i];
                        let changed = y >= ph
                            || x >= pw
                            || cell != &prev.cells[y as usize * pw as usize + x as usize];
                        if changed {
                            out.push_str(&format!("\x1b[{};{}H", y + 1, x + 1));
                            out.push_str(&cell.style.open());
                            out.push(cell.ch);
                            out.push_str(RESET);
                        }
                        x += 1;
                    }
                }
                // If the previous screen was taller, clear the leftover rows.
                if ph > h {
                    out.push_str(&format!("\x1b[{};1H\x1b[J", h + 1));
                }
            }
        }
        out
    }
}
