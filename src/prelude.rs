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

//! The prelude — import everything with `use ratatui_plus::prelude::*;`.

pub use crate::app::{App, Flow};
pub use crate::border::{
    self, ascii, block, curve, custom, dashed, dotted, double, round, thick, thin, BorderSet,
};
pub use crate::buffer::{Buffer, Cell};
pub use crate::canvas::Canvas;
pub use crate::color::{palette, Color, ColorLevel};
pub use crate::error::Rt;
pub use crate::event::{Event, Key, KeyCode, Mouse, MouseButton, MouseKind, Parser};
pub use crate::font;
pub use crate::frame::Frame;
pub use crate::layout::{Align, Direction, Point, Rect, Constraint, Layout};
pub use crate::style::Style;
pub use crate::widgets::{Ascii, Banner, Btn, Input, Panel, Text};
