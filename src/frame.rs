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

//! The per-frame drawing surface handed to your render callback.

use crate::buffer::Buffer;
use crate::canvas::Canvas;
use crate::color::ColorLevel;
use crate::layout::Rect;

/// A mutable view into the screen for one frame.
pub struct Frame<'a> {
    pub buf: &'a mut Buffer,
    /// The full screen area this frame may draw into.
    pub area: Rect,
    /// The detected color capability of the terminal (auto-degrade target).
    pub color_level: ColorLevel,
    /// Currently focused component index (keyboard navigation).
    focus: usize,
    /// Running counter of focusable components registered this frame.
    count: usize,
    /// A focus request raised by a click this frame.
    requested: Option<usize>,
}

impl<'a> Frame<'a> {
    pub(crate) fn new(buf: &'a mut Buffer, area: Rect, focus: usize, color_level: ColorLevel) -> Self {
        Frame {
            buf,
            area,
            color_level,
            focus,
            count: 0,
            requested: None,
        }
    }

    pub(crate) fn take_request(&mut self) -> Option<usize> {
        self.requested.take()
    }

    /// Register a focusable component; returns whether it is focused.
    /// Each call assigns the next sequential id.
    pub fn register_focus(&mut self) -> bool {
        let idx = self.count;
        self.count += 1;
        self.focus == idx || self.requested == Some(idx)
    }

    /// Request that the component with the given id receive focus
    /// (used when it is clicked).
    pub fn request_focus(&mut self, id: usize) {
        self.requested = Some(id);
    }

    /// The number of focusable components registered so far this frame.
    pub fn focus_count(&self) -> usize {
        self.count
    }

    /// A [`Canvas`] for low-level drawing into this frame's buffer.
    pub fn canvas(&mut self) -> Canvas<'_> {
        Canvas::new(self.buf)
    }
}
