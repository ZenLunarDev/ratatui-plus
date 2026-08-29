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

//! A live clock app.

use crate::desktop::{now_hms, DesktopApp};
use crate::prelude::*;

pub struct ClockApp;

pub fn clock_app() -> Box<dyn DesktopApp> {
    Box::new(ClockApp)
}

impl DesktopApp for ClockApp {
    fn title(&self) -> String {
        "Clock".into()
    }

    fn min_size(&self) -> (u16, u16) {
        (34, 14)
    }

    fn draw(&mut self, ui: &mut Frame, area: Rect) {
        let t = now_hms();
        let w = (t.len() as u16) * 6;
        let start = area.centered(w, 5).x;
        let mut x = start;
        for ch in t.chars() {
            Banner::new(&mut ui.buf, x, area.y + 2)
                .fg(Color::BrightCyan)
                .draw(&ch.to_string());
            x += 6;
        }
        Text::new(&mut ui.buf, Rect::new(area.x, area.y + 9, area.w, 1))
            .align(Align::Center)
            .fg(Color::BrightBlack)
            .draw("local time · auto-updates every tick");
    }
}
