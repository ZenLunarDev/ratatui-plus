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

//! A tiny windowing "desktop" — drag/resize/focus/close windows, a taskbar,
//! and an app launcher. This is what makes `ratatui-plus` feel like a full
//! terminal desktop environment (OpenTUI-style) while staying 100% std.

use crate::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// A program that can live inside a desktop [`Window`].
pub trait DesktopApp {
    /// Window title (shown in the title bar).
    fn title(&self) -> String;
    /// Render the app body into `area`.
    fn draw(&mut self, ui: &mut Frame, area: Rect);
    /// React to an event. Default: ignore.
    fn handle(&mut self, _ev: &Event, _ui: &mut Frame, _area: Rect) {}
    /// Minimum window size.
    fn min_size(&self) -> (u16, u16) {
        (28, 10)
    }
}

/// Factory that builds a fresh app instance (used by the launcher).
pub type AppFactory = fn() -> Box<dyn DesktopApp>;

const TITLE_H: u16 = 1;
const TASK_H: u16 = 1;
const TASK_BTN_W: u16 = 18;

/// A single window hosting a [`DesktopApp`].
struct Window {
    id: u32,
    rect: Rect,
    app: Box<dyn DesktopApp>,
    z: u32,
}

/// The desktop environment.
pub struct Desktop {
    windows: Vec<Window>,
    next_id: u32,
    focused: usize,
    drag: Option<(u32, i32, i32)>,
    resize: Option<(u32, u16, u16)>,
    bg: Color,
    show_taskbar: bool,
    launcher_open: bool,
    launcher_sel: usize,
    launcher_items: Vec<(&'static str, AppFactory)>,
    screen: Rect,
    quit: bool,
}

impl Desktop {
    /// Create a desktop with a default app set and the launcher open.
    pub fn new() -> Self {
        Desktop {
            windows: Vec::new(),
            next_id: 1,
            focused: 0,
            drag: None,
            resize: None,
            bg: Color::rgb(18, 18, 28),
            show_taskbar: true,
            launcher_open: true,
            launcher_sel: 0,
            launcher_items: vec![
                ("Clock", clock_app),
                ("Terminal", term_app),
                ("Snake", snake_app),
                ("Settings", settings_app),
            ],
            screen: Rect::new(0, 0, 80, 24),
            quit: false,
        }
    }

    /// Spawn a window for `app`, centered on the current screen.
    pub fn open(&mut self, app: Box<dyn DesktopApp>) {
        let (mw, mh) = app.min_size();
        let w = mw.max(28);
        let h = mh.max(10) + TITLE_H + 1;
        let r = self.center_rect(w, h);
        let id = self.next_id;
        self.next_id += 1;
        let mut win = Window {
            id,
            rect: r,
            app,
            z: id,
        };
        // cascade a little so stacked windows are distinguishable
        win.rect.x = win.rect.x.saturating_add((id as u16 % 6) * 2);
        win.rect.y = win.rect.y.saturating_add((id as u16 % 6) * 1);
        self.windows.push(win);
        self.focused = self.windows.len() - 1;
        self.bring_front(self.focused);
    }

    fn center_rect(&self, w: u16, h: u16) -> Rect {
        let w = w.min(self.screen.w);
        let h = h.min(self.screen.h.saturating_sub(TASK_H));
        Rect::new(
            self.screen.x + (self.screen.w - w) / 2,
            self.screen.y + (self.screen.h - h) / 2,
            w,
            h,
        )
    }

    fn bring_front(&mut self, i: usize) {
        let maxz = self.windows.iter().map(|w| w.z).max().unwrap_or(0) + 1;
        self.windows[i].z = maxz;
    }

    fn focus(&mut self, i: usize) {
        if i < self.windows.len() {
            self.focused = i;
            self.bring_front(i);
        }
    }

    fn close(&mut self, i: usize) {
        if i < self.windows.len() {
            self.windows.remove(i);
            if self.windows.is_empty() {
                self.focused = 0;
            } else {
                self.focused = self.focused.min(self.windows.len() - 1);
                self.bring_front(self.focused);
            }
        }
    }

    /// Run the desktop inside the real-time [`App`].
    pub fn run(mut self) -> Rt<()> {
        let app = App::new("ratatui-plus desktop")?
            .tab_cycle(false)
            .tick(std::time::Duration::from_millis(33));
        app.run(move |ui, ev| {
            self.screen = ui.area;
            self.step(ui, ev);
            if self.quit {
                Flow::Quit
            } else {
                Flow::Continue
            }
        })
    }

    fn step(&mut self, ui: &mut Frame, ev: &Event) {
        self.draw_bg(ui);
        let taskbar = if self.show_taskbar {
            Rect::new(0, ui.area.bottom().saturating_sub(TASK_H), ui.area.w, TASK_H)
        } else {
            Rect::new(0, ui.area.bottom(), 0, 0)
        };

        match ev {
            Event::Mouse(m) => self.on_mouse(ui, ev, m, taskbar),
            Event::Key(k) => self.on_key(ui, ev, k),
            _ => {}
        }

        self.draw_windows(ui);
        if self.show_taskbar {
            self.draw_taskbar(ui, taskbar);
        }
        if self.launcher_open {
            self.draw_launcher(ui);
        }
    }

    // ---- event handling -------------------------------------------------

    fn on_mouse(&mut self, ui: &mut Frame, ev: &Event, m: &Mouse, taskbar: Rect) {
        let (x, y) = (m.x, m.y);
        match m.kind {
            MouseKind::Press => {
                if self.show_taskbar && taskbar.contains(x, y) {
                    let idx = (x / TASK_BTN_W) as usize;
                    if idx < self.windows.len() {
                        self.focus(idx);
                    }
                    return;
                }
                if self.launcher_open && self.launcher_hit(ui, x, y) {
                    return;
                }
                if let Some(i) = self.top_window_at(x, y) {
                    let r = self.windows[i].rect;
                    if y == r.y && x >= r.right().saturating_sub(2) && x <= r.right().saturating_sub(1)
                    {
                        self.close(i);
                        return;
                    }
                    if y == r.y {
                        self.drag = Some((self.windows[i].id, x as i32 - r.x as i32, y as i32 - r.y as i32));
                        self.focus(i);
                        return;
                    }
                    if x == r.right().saturating_sub(1) && y == r.bottom().saturating_sub(1) {
                        self.resize = Some((self.windows[i].id, r.w, r.h));
                        self.focus(i);
                        return;
                    }
                    self.focus(i);
                    let area = self.windows[i].rect.inner();
                    let id = self.windows[i].id;
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                        w.app.handle(ev, ui, area);
                    }
                } else if self.launcher_open {
                    self.launcher_open = false;
                }
            }
            MouseKind::Motion => {
                if let Some((id, ox, oy)) = self.drag {
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                        let nx = (x as i32 - ox).max(0) as u16;
                        let ny = (y as i32 - oy).max(0) as u16;
                        w.rect.x = nx.min(ui.area.w.saturating_sub(3));
                        w.rect.y = ny.min(ui.area.h.saturating_sub(TASK_H).saturating_sub(1));
                    }
                }
                if let Some((id, _sw, _sh)) = self.resize {
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                        let (mn_w, mn_h) = w.app.min_size();
                        let nw = (x.saturating_sub(w.rect.x) + 1).max(mn_w);
                        let nh = (y.saturating_sub(w.rect.y) + 1).max(mn_h + TITLE_H + 1);
                        w.rect.w = nw.min(ui.area.w.saturating_sub(w.rect.x));
                        w.rect.h = nh.min(ui.area.h.saturating_sub(TASK_H).saturating_sub(w.rect.y));
                    }
                }
            }
            MouseKind::Release => {
                self.drag = None;
                self.resize = None;
            }
        }
    }

    fn on_key(&mut self, ui: &mut Frame, ev: &Event, k: &Key) {
        if k.is_ctrl('c') {
            return; // App handles quit
        }
        if self.launcher_open {
            match k.code {
                KeyCode::Up => self.launcher_sel = self.launcher_sel.saturating_sub(1),
                KeyCode::Down => {
                    self.launcher_sel = (self.launcher_sel + 1).min(self.launcher_items.len() - 1)
                }
                KeyCode::Enter | KeyCode::Char(' ') => self.launch_selected(),
                KeyCode::Esc => self.launcher_open = false,
                _ => {}
            }
            return;
        }
        match k.code {
            KeyCode::Tab => {
                if !self.windows.is_empty() {
                    self.focused = (self.focused + 1) % self.windows.len();
                    self.bring_front(self.focused);
                }
            }
            KeyCode::F(2) => self.launcher_open = true,
            KeyCode::Char('q') if k.ctrl => {
                if !self.windows.is_empty() {
                    let i = self.focused;
                    self.close(i);
                }
            }
            _ => {
                if !self.windows.is_empty() {
                    let i = self.focused;
                    let area = self.windows[i].rect.inner();
                    let id = self.windows[i].id;
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                        w.app.handle(ev, ui, area);
                    }
                }
            }
        }
    }

    fn launch_selected(&mut self) {
        if let Some((_, factory)) = self.launcher_items.get(self.launcher_sel).copied() {
            self.open(factory());
        }
        self.launcher_open = false;
    }

    fn top_window_at(&self, x: u16, y: u16) -> Option<usize> {
        let mut order: Vec<usize> = (0..self.windows.len()).collect();
        order.sort_by_key(|&i| self.windows[i].z);
        for &i in order.iter().rev() {
            if self.windows[i].rect.contains(x, y) {
                return Some(i);
            }
        }
        None
    }

    fn launcher_rect(&self) -> Rect {
        let w = 30u16;
        let h = (self.launcher_items.len() as u16) + 4;
        self.screen.centered(w, h)
    }

    fn launcher_hit(&mut self, _ui: &mut Frame, x: u16, y: u16) -> bool {
        let r = self.launcher_rect();
        if !r.contains(x, y) {
            return false;
        }
        let row = y.saturating_sub(r.y + 2);
        if row < self.launcher_items.len() as u16 {
            self.launcher_sel = row as usize;
            self.launch_selected();
        }
        true
    }

    // ---- rendering -------------------------------------------------------

    fn draw_bg(&mut self, ui: &mut Frame) {
        let screen = ui.area;
        let mut c = ui.canvas();
        c.fill_rect(screen, ' ', Style::new().bg(self.bg));
        drop(c);
        Text::new(&mut ui.buf, screen.centered(40, 1))
            .align(Align::Center)
            .fg(Color::rgb(60, 60, 80))
            .draw("ratatui-plus desktop");
        Text::new(&mut ui.buf, Rect::new(screen.x + 2, screen.y + 1, 30, 1))
            .fg(Color::rgb(50, 50, 70))
            .draw(&format!("color: {:?}", ui.color_level));
    }

    fn draw_windows(&mut self, ui: &mut Frame) {
        let mut order: Vec<usize> = (0..self.windows.len()).collect();
        order.sort_by_key(|&i| self.windows[i].z);
        for &i in &order {
            let focused = i == self.focused;
            self.draw_window(ui, i, focused);
        }
    }

    fn draw_window(&mut self, ui: &mut Frame, i: usize, focused: bool) {
        let r = self.windows[i].rect;
        let title = self.windows[i].app.title();
        let accent = if focused {
            Color::Cyan
        } else {
            Color::BrightBlack
        };
        {
            let mut c = ui.canvas().clip(r);
            c.border(r, border::thin(), Style::new().fg(accent));
        }
        // title bar
        let tstyle = if focused {
            Style::new().fg(Color::White).bg(Color::Blue)
        } else {
            Style::new().fg(Color::BrightBlack).bg(Color::Black)
        };
        let ttext = format!(" {} ", title);
        ui.buf.write_str(r.x + 1, r.y, &ttext, tstyle);
        ui.buf
            .put(r.right().saturating_sub(2), r.y, '×', Style::new().fg(Color::Red));
        // resize grip
        ui.buf
            .put(r.right().saturating_sub(1), r.bottom().saturating_sub(1), '╲', Style::new().fg(accent));
        // app body
        let area = r.inner();
        let id = self.windows[i].id;
        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
            w.app.draw(ui, area);
        }
    }

    fn draw_taskbar(&mut self, ui: &mut Frame, bar: Rect) {
        let st = Style::new().fg(Color::White).bg(Color::rgb(30, 30, 46));
        {
            let mut c = ui.canvas().clip(bar);
            c.fill_rect(bar, ' ', st);
        }
        for (i, win) in self.windows.iter().enumerate() {
            let x = bar.x + i as u16 * TASK_BTN_W;
            if x + TASK_BTN_W > bar.right() {
                break;
            }
            let active = i == self.focused;
            let lbl = format!("{}{}", if active { "▸" } else { " " }, win.app.title());
            let s = if active {
                Style::new().fg(Color::Black).bg(Color::Cyan)
            } else {
                st
            };
            ui.buf.write_str(x, bar.y, &lbl, s);
        }
        // clock on the right
        let clock = now_hms();
        let cx = bar.right().saturating_sub(clock.len() as u16 + 1);
        ui.buf.write_str(cx, bar.y, &clock, Style::new().fg(Color::BrightGreen).bg(Color::rgb(30, 30, 46)));
    }

    fn draw_launcher(&mut self, ui: &mut Frame) {
        let r = self.launcher_rect();
        {
            let mut c = ui.canvas().clip(r);
            c.border(r, border::double(), Style::new().fg(Color::Yellow));
        }
        Text::new(&mut ui.buf, Rect::new(r.x + 1, r.y + 1, r.w - 2, 1))
            .fg(Color::BrightYellow)
            .draw("◆ Launch an app");
        for (i, (name, _)) in self.launcher_items.iter().enumerate() {
            let sel = i == self.launcher_sel;
            let s = if sel {
                Style::new().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::new().fg(Color::White)
            };
            let row = r.y + 2 + i as u16;
            ui.buf.write_str(r.x + 2, row, name, s);
        }
        Text::new(&mut ui.buf, Rect::new(r.x + 1, r.bottom().saturating_sub(1), r.w - 2, 1))
            .fg(Color::BrightBlack)
            .draw("↑/↓ select · Enter open · F2 toggle");
    }
}

/// `HH:MM:SS` from the system clock (no external crates).
pub fn now_hms() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

// Fabric shims so the launcher can name the app modules without a hard
// dependency cycle at the call site. Real factories live in `apps`.
use crate::apps::{clock_app, settings_app, snake_app, term_app};
