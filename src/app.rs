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

//! The application runtime: real-time event loop, static rendering, and
//! focus management.

use crate::buffer::Buffer;
use crate::error::Rt;
use crate::event::{Event, Parser};
use crate::frame::Frame;
use crate::layout::Rect;
use crate::term::Term;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::thread;
use std::time::Duration;

/// Returned by the render callback to control the run loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    /// Keep rendering.
    Continue,
    /// Stop the loop and restore the terminal.
    Quit,
}

/// A terminal application.
pub struct App {
    term: Term,
    buf: Buffer,
    prev: Buffer,
    rx: Receiver<Event>,
    focus: usize,
    last_focus_count: usize,
    tick: Duration,
    running: bool,
}

impl App {
    /// Create a new app, open the terminal, and start the input reader.
    pub fn new(title: &str) -> Rt<Self> {
        let mut term = Term::new()?;
        let (w, h) = term.size();
        let area = Rect::new(0, 0, w, h);
        let buf = Buffer::empty(area);
        let prev = Buffer::empty(area);
        let (tx, rx) = channel::<Event>();

        #[cfg(unix)]
        let fd = term.input_fd();
        let tx_thread = tx.clone();

        thread::spawn(move || {
            let mut parser = Parser::new();
            let mut raw = [0u8; 1024];
            loop {
                #[cfg(unix)]
                let n = crate::term::read_tty(fd, &mut raw);
                #[cfg(windows)]
                let n = {
                    use std::io::Read;
                    let mut stdin = std::io::stdin();
                    match stdin.read(&mut raw) {
                        Ok(n) => n as isize,
                        Err(_) => 0,
                    }
                };
                if n <= 0 {
                    break;
                }
                let mut evs = Vec::new();
                parser.feed(&raw[..n as usize], &mut evs);
                for e in evs {
                    if tx_thread.send(e).is_err() {
                        return;
                    }
                }
            }
        });

        term.enable_raw()?;
        term.enable_alt()?;
        let _ = title;

        Ok(App {
            term,
            buf,
            prev,
            rx,
            focus: 0,
            last_focus_count: 0,
            tick: Duration::from_millis(33),
            running: true,
        })
    }

    /// Set the redraw/event tick rate.
    pub fn tick(mut self, d: Duration) -> Self {
        self.tick = d;
        self
    }

    /// Current terminal size.
    pub fn size(&self) -> (u16, u16) {
        self.term.size()
    }

    fn resize_buffers(&mut self, w: u16, h: u16) {
        let area = Rect::new(0, 0, w, h);
        self.buf.resize(area);
        self.prev = Buffer::empty(area);
    }

    fn draw_with<F>(&mut self, ev: &Event, cb: &mut F) -> Rt<Flow>
    where
        F: FnMut(&mut Frame, &Event) -> Flow,
    {
        self.buf.clear();
        let area = self.buf.area;
        let focus = self.focus;
        let mut frame = Frame::new(&mut self.buf, area, focus);
        let flow = cb(&mut frame, ev);
        if let Some(r) = frame.take_request() {
            self.focus = r;
        }
        self.last_focus_count = frame.focus_count();

        let out = self.buf.diff(Some(&self.prev));
        self.term.write_str(&out)?;
        self.prev = self.buf.clone();
        Ok(flow)
    }

    /// Run the real-time loop, calling `cb` for every event / tick.
    pub fn run<F>(mut self, mut cb: F) -> Rt<()>
    where
        F: FnMut(&mut Frame, &Event) -> Flow,
    {
        let (mut w, mut h) = self.term.size();
        loop {
            let ev = match self.rx.recv_timeout(self.tick) {
                Ok(e) => e,
                Err(RecvTimeoutError::Timeout) => Event::Tick,
                Err(RecvTimeoutError::Disconnected) => break,
            };

            // Ctrl-C always quits cleanly.
            if let Event::Key(k) = &ev {
                if k.is_ctrl('c') {
                    break;
                }
            }

            // Tab cycles focus, then we redraw to reflect it.
            if let Event::Key(k) = &ev {
                if k.is_tab() {
                    let n = self.last_focus_count.max(1);
                    self.focus = (self.focus + 1) % n;
                    let flow = self.draw_with(&Event::Tick, &mut cb)?;
                    if flow == Flow::Quit || !self.running {
                        break;
                    }
                    continue;
                }
            }

            // Detect terminal resize between frames.
            self.term.refresh_size();
            let (nw, nh) = self.term.size();
            if (nw, nh) != (w, h) {
                w = nw;
                h = nh;
                self.resize_buffers(w, h);
                let flow = self.draw_with(&Event::Resize(w, h), &mut cb)?;
                if flow == Flow::Quit {
                    break;
                }
                continue;
            }

            let flow = self.draw_with(&ev, &mut cb)?;
            if flow == Flow::Quit || !self.running {
                break;
            }
        }
        self.restore();
        Ok(())
    }

    /// Draw once and wait for Enter/Esc (static / non-animated mode).
    pub fn once<F>(mut self, mut cb: F) -> Rt<()>
    where
        F: FnMut(&mut Frame, &Event) -> Flow,
    {
        self.draw_with(&Event::Tick, &mut cb)?;
        loop {
            match self.rx.recv() {
                Ok(Event::Key(k)) if k.is_enter() || k.is_esc() || k.is_ctrl('c') => break,
                Ok(_) => {}
                Err(_) => break,
            }
        }
        self.restore();
        Ok(())
    }

    /// Convenience: build, draw once, wait for a key, and exit.
    pub fn static_ui<F>(title: &str, cb: F) -> Rt<()>
    where
        F: FnMut(&mut Frame, &Event) -> Flow,
    {
        App::new(title)?.once(cb)
    }

    fn restore(&mut self) {
        self.running = false;
        self.term.disable_alt();
        self.term.disable_raw();
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Ensure the terminal is always restored, even on panic.
        self.term.disable_alt();
        self.term.disable_raw();
    }
}
