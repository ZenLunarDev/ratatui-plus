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

//! Smoke tests for the pure (non-terminal) logic.

use ratatui_plus::prelude::*;

#[test]
fn color_hex_and_rgb() {
    assert_eq!(Color::hex("#ff0000"), Color::rgb(255, 0, 0));
    assert_eq!(Color::hex("#0f0"), Color::rgb(0, 255, 0));
    let g = Color::Red.gradient(Color::Blue, 3);
    assert_eq!(g.len(), 3);
    assert_eq!(g[0].to_rgb(), (204, 0, 0));
    assert_eq!(g[2].to_rgb(), (0, 0, 204));
}

#[test]
fn rect_helpers() {
    let r = Rect::new(0, 0, 10, 10);
    assert!(r.contains(5, 5));
    assert!(!r.contains(10, 10));
    let p = r.padding(2);
    assert_eq!(p, Rect::new(2, 2, 6, 6));
    let cols = r.split_v(2, 1);
    assert_eq!(cols.len(), 2);
}

#[test]
fn parser_keys() {
    let mut p = Parser::new();
    let mut out = Vec::new();
    p.feed(b"a", &mut out);
    assert_eq!(out.len(), 1);
    match &out[0] {
        Event::Key(k) => assert_eq!(k.code, KeyCode::Char('a')),
        _ => panic!("expected key"),
    }

    let mut p = Parser::new();
    let mut out = Vec::new();
    p.feed(b"\x1b[A", &mut out);
    assert_eq!(out[0], Event::Key(Key::new(KeyCode::Up)));

    let mut p = Parser::new();
    let mut out = Vec::new();
    p.feed(b"\x03", &mut out);
    assert!(matches!(&out[0], Event::Key(k) if k.is_ctrl('c')));
}

#[test]
fn buffer_diff_emits() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 5, 1));
    buf.put(0, 0, 'X', Style::new().fg(Color::Red));
    let out = buf.diff(None);
    assert!(out.contains('X'));
    assert!(out.contains("31")); // fg red code
}

#[test]
fn banner_glyph_width() {
    assert_eq!(font::GLYPH_W, 5);
    assert!(font::glyph('A').is_some());
    assert!(font::glyph('Z').is_some());
    assert!(font::glyph('7').is_some());
}
