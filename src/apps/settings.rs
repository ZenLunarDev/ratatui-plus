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

//! A settings panel that shows the auto-detected terminal capabilities.

use crate::desktop::DesktopApp;
use crate::prelude::*;

pub struct SettingsApp;

pub fn settings_app() -> Box<dyn DesktopApp> {
    Box::new(SettingsApp)
}

impl DesktopApp for SettingsApp {
    fn title(&self) -> String {
        "Settings".into()
    }

    fn min_size(&self) -> (u16, u16) {
        (40, 16)
    }

    fn draw(&mut self, ui: &mut Frame, area: Rect) {
        let lvl = match ui.color_level {
            ColorLevel::Rgb => "truecolor (24-bit)",
            ColorLevel::Ansi256 => "256-color",
            ColorLevel::Basic => "16-color",
            ColorLevel::None => "no-color",
        };
        let lines = format!(
            "color level : {lvl}\n\
             std only    : yes (no external crates)\n\
             mode        : realtime + static\n\
             auto-size   : responsive Layout engine\n\
             transparency : alpha on supported terminals"
        );
        Text::new(&mut ui.buf, Rect::new(area.x + 1, area.y + 1, area.w - 2, 6))
            .fg(Color::White)
            .draw(&lines);

        // transparency demo bar
        let bar = Rect::new(area.x + 1, area.y + 9, area.w - 2, 2);
        let mut c = ui.canvas().clip(bar);
        c.fill_rect(bar, ' ', Style::new().fg(Color::Black).bg(Color::Magenta).alpha(0.5));
        Text::new(&mut ui.buf, Rect::new(area.x + 1, area.y + 9, area.w - 2, 1))
            .fg(Color::White)
            .draw("   translucent panel (alpha 0.5)");
    }
}
