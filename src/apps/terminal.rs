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

//! A tiny interactive terminal emulator (echo + history).

use crate::desktop::DesktopApp;
use crate::prelude::*;

pub struct TermApp {
    hist: Vec<String>,
    input: Input,
}

pub fn term_app() -> Box<dyn DesktopApp> {
    Box::new(TermApp {
        hist: vec!["ratatui-plus terminal — type and press Enter".into()],
        input: Input::new(Rect::new(0, 0, 30, 1))
            .hint("command")
            .max(200),
    })
}

impl DesktopApp for TermApp {
    fn title(&self) -> String {
        "Terminal".into()
    }

    fn min_size(&self) -> (u16, u16) {
        (44, 14)
    }

    fn handle(&mut self, ev: &Event, ui: &mut Frame, area: Rect) {
        let bar = Rect::new(area.x, area.bottom().saturating_sub(1), area.w, 1);
        self.input.set_rect(bar);
        self.input.handle(ev, ui);
        if self.input.submitted() {
            let line = self.input.value().to_string();
            if !line.is_empty() {
                self.hist.push(format!("{}", line));
            }
            self.input.clear();
        }
    }

    fn draw(&mut self, ui: &mut Frame, area: Rect) {
        let bar = Rect::new(area.x, area.bottom().saturating_sub(1), area.w, 1);
        self.input.set_rect(bar);

        let max_lines = area.h.saturating_sub(1) as usize;
        let start = self.hist.len().saturating_sub(max_lines);
        let mut y = area.y;
        for line in &self.hist[start..] {
            Text::new(&mut ui.buf, Rect::new(area.x, y, area.w, 1))
                .fg(Color::BrightGreen)
                .draw(line);
            y += 1;
            if y >= bar.y {
                break;
            }
        }
        self.input.draw(ui);
    }
}
