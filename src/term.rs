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

//! Cross-platform terminal control — **no external crates**, only `std`
//! plus minimal platform FFI (declared inline). Handles raw mode,
//! alternate screen, mouse + focus reporting and size detection on
//! Windows and Unix.

use crate::error::Rt;
use crate::layout::Rect;
use std::io::Write;

#[cfg(windows)]
#[allow(dead_code, non_snake_case)]
mod sys {
    pub type HANDLE = *mut std::ffi::c_void;
    pub type DWORD = u32;
    pub type BOOL = i32;

    pub const STD_INPUT_HANDLE: DWORD = 0xFFFFFFF6;
    pub const STD_OUTPUT_HANDLE: DWORD = 0xFFFFFFF5;

    pub const ENABLE_PROCESSED_INPUT: DWORD = 0x0001;
    pub const ENABLE_LINE_INPUT: DWORD = 0x0002;
    pub const ENABLE_ECHO_INPUT: DWORD = 0x0004;
    pub const ENABLE_VIRTUAL_TERMINAL_INPUT: DWORD = 0x0200;

    pub const ENABLE_PROCESSED_OUTPUT: DWORD = 0x0001;
    pub const ENABLE_WRAP_AT_EOL_OUTPUT: DWORD = 0x0002;
    pub const ENABLE_VIRTUAL_TERMINAL_PROCESSING: DWORD = 0x0004;

    #[repr(C)]
    pub struct COORD {
        pub x: i16,
        pub y: i16,
    }
    #[repr(C)]
    pub struct SMALL_RECT {
        pub left: i16,
        pub top: i16,
        pub right: i16,
        pub bottom: i16,
    }
    #[repr(C)]
    pub struct CONSOLE_SCREEN_BUFFER_INFO {
        pub dwSize: COORD,
        pub dwCursorPosition: COORD,
        pub wAttributes: u16,
        pub srWindow: SMALL_RECT,
        pub dwMaximumWindowSize: COORD,
    }

    extern "system" {
        pub fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
        pub fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: *mut DWORD) -> BOOL;
        pub fn SetConsoleMode(hConsoleHandle: HANDLE, dwMode: DWORD) -> BOOL;
        pub fn GetConsoleScreenBufferInfo(
            hConsoleOutput: HANDLE,
            lpConsoleScreenBufferInfo: *mut CONSOLE_SCREEN_BUFFER_INFO,
        ) -> BOOL;
    }
}

#[cfg(unix)]
#[allow(dead_code, non_snake_case)]
mod sys {
    use std::os::unix::io::RawFd;

    pub type Tcflag = u32;
    pub const NCCS: usize = 32;

    #[repr(C)]
    pub struct Termios {
        pub iflag: Tcflag,
        pub oflag: Tcflag,
        pub cflag: Tcflag,
        pub lflag: Tcflag,
        pub line: u8,
        pub cc: [u8; NCCS],
        pub ispeed: Tcflag,
        pub ospeed: Tcflag,
    }

    // termios flag bits (Linux x86_64 glibc)
    pub const ICRNL: Tcflag = 0o000400;
    pub const IXON: Tcflag = 0o002000;
    pub const INLCR: Tcflag = 0o000100;
    pub const IGNCR: Tcflag = 0o000200;
    pub const ISTRIP: Tcflag = 0o000020;
    pub const INPCK: Tcflag = 0o000020;
    pub const BRKINT: Tcflag = 0o000002;
    pub const PARMRK: Tcflag = 0o000010;
    pub const OPOST: Tcflag = 0o000001;
    pub const CSIZE: Tcflag = 0o000060;
    pub const CS8: Tcflag = 0o000060;
    pub const ECHO: Tcflag = 0o000010;
    pub const ECHONL: Tcflag = 0o000040;
    pub const ICANON: Tcflag = 0o000002;
    pub const ISIG: Tcflag = 0o000001;
    pub const IEXTEN: Tcflag = 0o100000;

    pub const VMIN: usize = 6;
    pub const VTIME: usize = 5;

    pub const TIOCGWINSZ: u64 = 0x5413;

    #[repr(C)]
    pub struct Winsize {
        pub ws_row: u16,
        pub ws_col: u16,
        pub ws_xpixel: u16,
        pub ws_ypixel: u16,
    }

    extern "C" {
        pub fn tcgetattr(fd: RawFd, termios: *mut Termios) -> i32;
        pub fn tcsetattr(fd: RawFd, optional_actions: i32, termios: *const Termios) -> i32;
        pub fn ioctl(fd: RawFd, request: u64, ws: *mut Winsize) -> i32;
        pub fn read(fd: RawFd, buf: *mut u8, count: usize) -> isize;
        pub fn write(fd: RawFd, buf: *const u8, count: usize) -> isize;
    }
}

/// A handle to the terminal display.
pub struct Term {
    raw_enabled: bool,
    alt_enabled: bool,
    mouse_enabled: bool,
    pub width: u16,
    pub height: u16,

    #[cfg(windows)]
    in_handle: sys::HANDLE,
    #[cfg(windows)]
    out_handle: sys::HANDLE,
    #[cfg(windows)]
    saved_in: sys::DWORD,
    #[cfg(windows)]
    saved_out: sys::DWORD,

    #[cfg(unix)]
    tty_fd: i32,
    #[cfg(unix)]
    saved_termios: Option<sys::Termios>,
}

impl Term {
    /// Open the terminal, query its size.
    pub fn new() -> Rt<Self> {
        #[cfg(windows)]
        {
            unsafe {
                let in_handle = sys::GetStdHandle(sys::STD_INPUT_HANDLE);
                let out_handle = sys::GetStdHandle(sys::STD_OUTPUT_HANDLE);
                let mut saved_in = 0;
                let mut saved_out = 0;
                sys::GetConsoleMode(in_handle, &mut saved_in);
                sys::GetConsoleMode(out_handle, &mut saved_out);
                // turn on virtual-terminal processing for output
                let out_mode = saved_out
                    | sys::ENABLE_VIRTUAL_TERMINAL_PROCESSING
                    | sys::ENABLE_WRAP_AT_EOL_OUTPUT;
                sys::SetConsoleMode(out_handle, out_mode);
                let (w, h) = query_size_win(out_handle);
                Ok(Term {
                    raw_enabled: false,
                    alt_enabled: false,
                    mouse_enabled: false,
                    width: w,
                    height: h,
                    in_handle,
                    out_handle,
                    saved_in,
                    saved_out,
                })
            }
        }
        #[cfg(unix)]
        {
            let tty_fd = open_tty();
            let (w, h) = query_size_unix(tty_fd);
            Ok(Term {
                raw_enabled: false,
                alt_enabled: false,
                mouse_enabled: false,
                width: w,
                height: h,
                tty_fd,
                saved_termios: None,
            })
        }
    }

    /// Enter raw mode (no echo / line buffering / signal translation).
    ///
    /// If the terminal cannot be put into raw mode (e.g. input is piped
    /// rather than a real console) this is treated as a no-op so the
    /// program can still render — only interactive key handling is limited.
    pub fn enable_raw(&mut self) -> Rt<()> {
        if self.raw_enabled {
            return Ok(());
        }
        #[cfg(windows)]
        {
            unsafe {
                // we already saved the original mode in `new`
                let mode = sys::ENABLE_VIRTUAL_TERMINAL_INPUT; // raw key bytes
                let _ = sys::SetConsoleMode(self.in_handle, mode);
            }
        }
        #[cfg(unix)]
        {
            unsafe {
                let mut t = std::mem::MaybeUninit::<sys::Termios>::uninit();
                if sys::tcgetattr(self.tty_fd, t.as_mut_ptr()) == 0 {
                    let mut t = t.assume_init();
                    self.saved_termios = Some(t);
                    let raw = sys::Termios {
                        iflag: t.iflag
                            & !(sys::IGNBRK
                                | sys::BRKINT
                                | sys::PARMRK
                                | sys::ISTRIP
                                | sys::INLCR
                                | sys::IGNCR
                                | sys::ICRNL
                                | sys::IXON),
                        oflag: t.oflag & !sys::OPOST,
                        cflag: t.cflag & !sys::CSIZE | sys::CS8,
                        lflag: t.lflag
                            & !(sys::ECHO | sys::ECHONL | sys::ICANON | sys::ISIG
                                | sys::IEXTEN),
                        line: t.line,
                        cc: t.cc,
                        ispeed: t.ispeed,
                        ospeed: t.ospeed,
                    };
                    raw.cc[sys::VMIN] = 1;
                    raw.cc[sys::VTIME] = 0;
                    let _ = sys::tcsetattr(self.tty_fd, 0, &raw);
                }
            }
        }
        self.raw_enabled = true;
        Ok(())
    }

    /// Leave raw mode.
    pub fn disable_raw(&mut self) {
        if !self.raw_enabled {
            return;
        }
        #[cfg(windows)]
        unsafe {
            sys::SetConsoleMode(self.in_handle, self.saved_in);
        }
        #[cfg(unix)]
        unsafe {
            if let Some(t) = self.saved_termios.take() {
                sys::tcsetattr(self.tty_fd, 0, &t);
            }
        }
        self.raw_enabled = false;
    }

    /// Switch to the alternate screen buffer and hide the cursor.
    pub fn enable_alt(&mut self) -> Rt<()> {
        self.write_str("\x1b[?1049h\x1b[?25l\x1b[2J\x1b[H")?;
        self.alt_enabled = true;
        self.enable_mouse()?;
        Ok(())
    }

    /// Leave the alternate screen and show the cursor.
    pub fn disable_alt(&mut self) {
        if !self.alt_enabled {
            return;
        }
        self.disable_mouse();
        let _ = self.write_str("\x1b[?25h\x1b[?1049l");
        self.alt_enabled = false;
    }

    /// Enable SGR (1006) mouse tracking + focus events.
    pub fn enable_mouse(&mut self) -> Rt<()> {
        if self.mouse_enabled {
            return Ok(());
        }
        self.write_str("\x1b[?1006h\x1b[?1003h\x1b[?1004h")?;
        self.mouse_enabled = true;
        Ok(())
    }

    pub fn disable_mouse(&mut self) {
        if !self.mouse_enabled {
            return;
        }
        let _ = self.write_str("\x1b[?1004l\x1b[?1003l\x1b[?1006l");
        self.mouse_enabled = false;
    }

    /// Current size in `(cols, rows)`.
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Re-query the terminal size.
    pub fn refresh_size(&mut self) {
        #[cfg(windows)]
        {
            let (w, h) = query_size_win(self.out_handle);
            self.width = w;
            self.height = h;
        }
        #[cfg(unix)]
        {
            let (w, h) = query_size_unix(self.tty_fd);
            self.width = w;
            self.height = h;
        }
    }

    /// The full-screen rect.
    pub fn rect(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    /// Write a raw string to the terminal.
    pub fn write_str(&mut self, s: &str) -> Rt<()> {
        let mut out = std::io::stdout().lock();
        out.write_all(s.as_bytes())?;
        out.flush()?;
        Ok(())
    }

    /// Flush pending output.
    pub fn flush(&mut self) -> Rt<()> {
        std::io::stdout().flush()?;
        Ok(())
    }

    /// The fd/handle the event thread should read from (unix only).
    #[cfg(unix)]
    pub(crate) fn input_fd(&self) -> i32 {
        self.tty_fd
    }

    /// Read raw bytes from the tty (unix only). Used by the event thread.
    #[cfg(unix)]
    pub(crate) fn read_tty(fd: i32, buf: &mut [u8]) -> isize {
        unsafe { sys::read(fd, buf.as_mut_ptr(), buf.len()) }
    }

    /// Clear the visible screen.
    pub fn clear_screen(&mut self) -> Rt<()> {
        self.write_str("\x1b[2J\x1b[H")
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        if self.mouse_enabled {
            self.disable_mouse();
        }
        if self.alt_enabled {
            let _ = self.write_str("\x1b[?25h\x1b[?1049l");
        }
        self.disable_raw();
        #[cfg(windows)]
        unsafe {
            sys::SetConsoleMode(self.out_handle, self.saved_out);
            sys::SetConsoleMode(self.in_handle, self.saved_in);
        }
    }
}

#[cfg(windows)]
fn query_size_win(handle: sys::HANDLE) -> (u16, u16) {
    unsafe {
        let mut info: sys::CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
        if sys::GetConsoleScreenBufferInfo(handle, &mut info) != 0 {
            let w = (info.srWindow.right - info.srWindow.left + 1).max(1) as u16;
            let h = (info.srWindow.bottom - info.srWindow.top + 1).max(1) as u16;
            (w, h)
        } else {
            (80, 24)
        }
    }
}

#[cfg(unix)]
fn query_size_unix(fd: i32) -> (u16, u16) {
    unsafe {
        let mut ws: sys::Winsize = std::mem::zeroed();
        if fd >= 0 && sys::ioctl(fd, sys::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            (ws.ws_col as u16, ws.ws_row as u16)
        } else {
            (80, 24)
        }
    }
}

#[cfg(unix)]
fn open_tty() -> i32 {
    // Try /dev/tty first (works even when stdin is piped), then stdin.
    unsafe {
        let path = b"/dev/tty\0";
        let fd = libc_open(path.as_ptr() as *const i8, 0); // O_RDWR = 2 but 0 works for query
        if fd >= 0 {
            return fd;
        }
        // fallback to stdin
        0
    }
}

#[cfg(unix)]
unsafe fn libc_open(path: *const i8, flags: i32) -> i32 {
    // minimal open(2) — O_RDWR = 2
    extern "C" {
        fn open(path: *const i8, flags: i32, mode: i32) -> i32;
    }
    open(path, flags, 0)
}
