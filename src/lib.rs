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

//! # ratatui-plus
//!
//! A feature-rich, **100% standard-library** Terminal UI toolkit by
//! `ZenLunarDev`. Borders & curves, deep true-color + 256-color + 4-bit
//! palettes, transparency, ASCII/banner fonts, and interactive widgets
//! (button, text input, text) — with a short, easy-to-remember API.
//!
//! Two rendering modes:
//! * **Real-time** — [`App::run`] drives an event loop that redraws on
//!   every key, mouse, resize or tick. Great for live dashboards, games
//!   and editors.
//! * **Static** — [`App::once`] / [`App::static_ui`] draws a single frame
//!   and waits for a key. Great for menus and reports.
//!
//! ```no_run
//! use ratatui_plus::prelude::*;
//!
//! fn main() -> Rt {
//!     let mut app = App::new("demo")?;
//!     app.run(|ui, ev| {
//!         let r = ui.area;
//!         Panel::new(&mut ui.buf, r.padding(2))
//!             .round()
//!             .fg(Color::Cyan)
//!             .title("Hello")
//!             .draw();
//!         match ev {
//!             Event::Key(k) if k.is_esc() => Flow::Quit,
//!             _ => Flow::Continue,
//!         }
//!     })?;
//!     Ok(())
//! }
//! ```
//!
//! Browse the [`prelude`] for the full, memorable vocabulary.

pub mod app;
pub mod border;
pub mod buffer;
pub mod canvas;
pub mod color;
pub mod error;
pub mod event;
pub mod font;
pub mod frame;
pub mod layout;
pub mod style;
pub mod term;
pub mod widgets;
pub mod prelude;

