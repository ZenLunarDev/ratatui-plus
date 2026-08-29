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

//! A feature-packed, responsive demo of `ratatui-plus`.

use ratatui_plus::prelude::*;

struct State {
    name: Input,
    go: Btn,
    clicks: u32,
    ticks: u64,
}

fn main() -> Rt {
    let mut state = State {
        name: Input::new(Rect::new(0, 0, 24, 1)).hint("type your name").max(20),
        go: Btn::new(Rect::new(0, 0, 14, 3)).label("CLICK!").round(),
        clicks: 0,
        ticks: 0,
    };

    let app = App::new("ratatui-plus demo")?.tick(std::time::Duration::from_millis(33));

    app.run(move |ui, ev| {
        let screen = ui.area;
        if let Event::Tick = ev {
            state.ticks += 1;
        }

        // --- Responsive auto-size layout (header / body / footer) ---
        let chunks = Layout::vertical([
            Constraint::Length(9), // header
            Constraint::Fill,      // body (takes the rest)
            Constraint::Length(3), // footer
        ])
        .margin(1)
        .split(screen);
        let [header, body, footer] = [chunks[0], chunks[1], chunks[2]];

        // --- Header: gradient banner, auto-centered ---
        let title = "RATATUI+PLUS";
        let grad = Color::Magenta.gradient(Color::Cyan, title.len());
        let total_w = (title.len() as u16) * 6;
        let start_x = header.centered(total_w, 1).x;
        let mut x = start_x;
        for (i, ch) in title.chars().enumerate() {
            Banner::new(&mut ui.buf, x, header.y + 1).fg(grad[i]).draw(&ch.to_string());
            x += 6;
        }
        Text::new(&mut ui.buf, Rect::new(header.x, header.y + 7, header.w, 1))
            .align(Align::Center)
            .fg(Color::BrightBlack)
            .draw("std-only  ::  auto-size  ::  auto-color  ::  realtime + static");

        // --- Body split into two auto-sized columns ---
        let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .spacing(1)
            .split(body);
        let left = cols[0];
        let right = cols[1];

        // Left panel with curved border + canvas shapes.
        Panel::new(&mut ui.buf, left)
            .round()
            .fg(Color::Green)
            .title("graphics")
            .draw();
        let li = left.inner();
        let mut c = ui.canvas().clip(left);
        c.circle(
            li.x as i32 + 10,
            li.y as i32 + 6,
            5,
            '●',
            Style::new().fg(Color::BrightCyan).alpha(0.6),
        );
        c.curve(
            Point::new(li.x, li.bottom()),
            Point::new(li.x + li.w / 2, li.bottom().saturating_add(4)),
            Point::new(li.x + li.w, li.bottom()),
            '~',
            Style::new().fg(Color::BrightMagenta),
        );
        let bx = li.x as i32 + 20 + (state.ticks % 18) as i32;
        let by = li.y as i32 + 2 + ((state.ticks / 4) % 6) as i32;
        c.put(bx, by, '◉', Style::new().fg(Color::BrightYellow).blink(true));

        // Right panel with interactive widgets.
        Panel::new(&mut ui.buf, right)
            .thick()
            .fg(Color::Yellow)
            .title("input")
            .draw();
        let ri = right.inner();
        state.name.set_rect(Rect::new(ri.x, ri.y + 1, ri.w.min(24), 1));
        state.name.handle(ev, ui);
        state.name.draw(ui);
        Text::new(&mut ui.buf, Rect::new(ri.x, ri.y, ri.w, 1))
            .fg(Color::BrightBlack)
            .draw("your name:");
        state.go.set_rect(Rect::new(ri.x, ri.y + 4, 14, 3));
        state.go.handle(ev, ui);
        state.go.draw(ui);
        if state.go.clicked() {
            state.clicks += 1;
        }
        let art = "  /\\_/\\\n ( o.o )\n  > ^ <";
        Ascii::new(&mut ui.buf, ri.x, ri.y + 9)
            .fg(Color::rgb(255, 105, 180))
            .draw(art);

        // --- Footer: transparency + detected color level ---
        let lvl = match ui.color_level {
            ColorLevel::Rgb => "truecolor",
            ColorLevel::Ansi256 => "256-color",
            ColorLevel::Basic => "16-color",
            ColorLevel::None => "no-color",
        };
        let msg = format!(
            "clicks: {}  name: '{}'  color: {}  [Tab] focus  [Esc] quit",
            state.clicks,
            state.name.value(),
            lvl
        );
        Text::new(&mut ui.buf, footer)
            .style(Style::new().fg(Color::Black).bg(Color::Yellow).alpha(0.85))
            .draw(&msg);

        match ev {
            Event::Key(k) if k.is_esc() || k.is_ctrl('c') => Flow::Quit,
            _ => Flow::Continue,
        }
    })?;

    Ok(())
}
