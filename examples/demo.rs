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

//! A feature-packed demo of `ratatui-plus`.

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

        // --- Big gradient banner title (one letter per color) ---
        let title = "RATATUI";
        let grad = Color::Magenta.gradient(Color::Cyan, title.len());
        let mut x = screen.x + 2;
        for (i, ch) in title.chars().enumerate() {
            Banner::new(&mut ui.buf, x, screen.y + 1).fg(grad[i]).draw(&ch.to_string());
            x += 6;
        }
        Banner::new(&mut ui.buf, screen.x + 2, screen.y + 7)
            .fg(Color::rgb(255, 215, 0))
            .draw("+ PLUS");

        // --- Rounded feature panel ---
        let panel = screen.padding(2).inset(0, 12);
        Panel::new(&mut ui.buf, panel)
            .round()
            .fg(Color::Green)
            .title("features")
            .draw();
        let inner = panel.inner();

        let mut c = ui.canvas().clip(panel);
        // a curve + a translucent circle, drawn with the canvas
        c.circle(
            inner.x as i32 + 10,
            inner.y as i32 + 6,
            5,
            '●',
            Style::new().fg(Color::BrightCyan).alpha(0.6),
        );
        c.curve(
            Point::new(inner.x, inner.bottom()),
            Point::new(inner.x + inner.w / 2, inner.bottom().saturating_add(4)),
            Point::new(inner.x + inner.w, inner.bottom()),
            '~',
            Style::new().fg(Color::BrightMagenta),
        );
        // animated bouncing dot
        let bx = inner.x as i32 + 20 + (state.ticks % 20) as i32;
        let by = inner.y as i32 + 2 + ((state.ticks / 4) % 6) as i32;
        c.put(bx, by, '◉', Style::new().fg(Color::BrightYellow).blink(true));

        Text::new(&mut ui.buf, Rect::new(inner.x + 30, inner.y + 2, 40, 8))
            .fg(Color::White)
            .draw(
                "std-only  :: no external crates\n\
                 colors   :: truecolor + 256 + 4bit\n\
                 borders  :: round curve thick ascii\n\
                 widgets  :: button input text ascii\n\
                 mode     :: realtime + static",
            );

        // --- Interactive widgets (repositioned, state preserved) ---
        let iy = screen.bottom().saturating_sub(4);
        state.name.set_rect(Rect::new(screen.x + 2, iy, 24, 1));
        state.name.handle(ev, ui);
        state.name.draw(ui);

        state.go.set_rect(Rect::new(screen.x + 30, iy, 14, 3));
        state.go.handle(ev, ui);
        state.go.draw(ui);

        if state.go.clicked() {
            state.clicks += 1;
        }

        // --- Footer with transparency ---
        let foot = Rect::new(screen.x + 2, screen.bottom().saturating_sub(2), screen.w - 4, 1);
        let msg = format!(
            "clicks: {}  name: '{}'   [Tab] focus  [Esc] quit",
            state.clicks,
            state.name.value()
        );
        Text::new(&mut ui.buf, foot)
            .style(Style::new().fg(Color::Black).bg(Color::Yellow).alpha(0.85))
            .draw(&msg);

        match ev {
            Event::Key(k) if k.is_esc() || k.is_ctrl('c') => Flow::Quit,
            _ => Flow::Continue,
        }
    })?;

    Ok(())
}
