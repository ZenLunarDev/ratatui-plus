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

//! A playable snake game on the canvas.

use crate::desktop::DesktopApp;
use crate::prelude::*;

pub struct SnakeApp {
    body: Vec<(u16, u16)>,
    dir: (i16, i16),
    food: (u16, u16),
    acc: u64,
    dead: bool,
    score: u32,
}

pub fn snake_app() -> Box<dyn DesktopApp> {
    Box::new(SnakeApp::new())
}

impl SnakeApp {
    fn new() -> Self {
        SnakeApp {
            body: vec![(5, 5), (4, 5), (3, 5)],
            dir: (1, 0),
            food: (10, 8),
            acc: 0,
            dead: false,
            score: 0,
        }
    }

    fn step(&mut self, area: Rect) {
        if self.dead {
            return;
        }
        let p = area.inner();
        let (hx, hy) = self.body[0];
        let nx = hx as i16 + self.dir.0;
        let ny = hy as i16 + self.dir.1;
        if nx < 0 || ny < 0 || nx >= p.w as i16 || ny >= p.h as i16 {
            self.dead = true;
            return;
        }
        let (nx, ny) = (nx as u16, ny as u16);
        if self.body.iter().any(|&(x, y)| x == nx && y == ny) {
            self.dead = true;
            return;
        }
        if (nx, ny) == self.food {
            self.score += 1;
            self.body.insert(0, (nx, ny));
            let fw = p.w.max(1);
            let fh = p.h.max(1);
            self.food = (self.acc as u16 % fw, (self.acc / 7) as u16 % fh);
        } else {
            self.body.insert(0, (nx, ny));
            self.body.pop();
        }
    }
}

impl DesktopApp for SnakeApp {
    fn title(&self) -> String {
        "Snake".into()
    }

    fn min_size(&self) -> (u16, u16) {
        (30, 16)
    }

    fn handle(&mut self, ev: &Event, _ui: &mut Frame, area: Rect) {
        match ev {
            Event::Key(k) => match k.code {
                KeyCode::Up if self.dir.1 == 0 => self.dir = (0, -1),
                KeyCode::Down if self.dir.1 == 0 => self.dir = (0, 1),
                KeyCode::Left if self.dir.0 == 0 => self.dir = (-1, 0),
                KeyCode::Right if self.dir.0 == 0 => self.dir = (1, 0),
                _ => {}
            },
            Event::Tick => {
                self.acc += 1;
                if self.acc % 4 == 0 {
                    self.step(area);
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, ui: &mut Frame, area: Rect) {
        let p = area.inner();
        let mut c = ui.canvas().clip(p);
        c.put(
            p.x as i32 + self.food.0 as i32,
            p.y as i32 + self.food.1 as i32,
            '★',
            Style::new().fg(Color::Yellow),
        );
        for (x, y) in &self.body {
            c.put(
                p.x as i32 + *x as i32,
                p.y as i32 + *y as i32,
                '█',
                Style::new().fg(Color::BrightGreen),
            );
        }
        Text::new(&mut ui.buf, Rect::new(p.x, p.y, p.w, 1))
            .fg(Color::BrightBlack)
            .draw(&format!("score: {}  (arrows to move)", self.score));
        if self.dead {
            Text::new(&mut ui.buf, p.centered(12, 1))
                .fg(Color::Red)
                .draw("GAME OVER");
        }
    }
}
