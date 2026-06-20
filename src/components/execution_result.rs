use ansi_to_tui::IntoText;
use color_eyre::eyre::Result;
use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::{buffer::CellWidth as _, prelude::*, widgets::*};
use symbols::scrollbar;
use unicode_segmentation::UnicodeSegmentation as _;

use super::{Component, Frame};
use crate::{
    action::Action,
    config::Config,
    termtext::{Char, Text},
};

pub struct ExecutionResult {
    config: Config,

    result: Option<Text>,

    x_position: u16,
    y_position: u16,
    y_area_size: u16,
    y_max_scroll_size: u16,
    fold: bool,

    rect: Rect,
}

impl ExecutionResult {
    pub fn new(config: Config, fold: bool) -> Self {
        Self {
            config,
            result: None,
            fold,
            y_area_size: 0,
            y_max_scroll_size: 0,
            x_position: 0,
            y_position: 0,
            rect: Rect::default(),
        }
    }

    fn set_result(&mut self, new: Option<Text>) {
        self.result = new;
    }

    fn scroll_down(&mut self) {
        self.y_position = self.y_position.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.y_position = self.y_position.saturating_sub(1);
    }

    fn page_up(&mut self) {
        self.y_position = self.y_position.saturating_sub(self.y_area_size);
    }

    fn page_down(&mut self) {
        self.y_position = self.y_position.saturating_add(self.y_area_size);
    }

    fn half_page_up(&mut self) {
        self.y_position = self.y_position.saturating_sub(self.y_area_size / 2);
    }

    fn half_page_down(&mut self) {
        self.y_position = self.y_position.saturating_add(self.y_area_size / 2);
    }

    fn bottom_of_page(&mut self) {
        self.y_position = self.y_max_scroll_size;
    }

    fn top_of_page(&mut self) {
        self.y_position = 0;
    }

    fn scroll_right(&mut self) {
        self.x_position = self.x_position.saturating_add(10);
    }

    fn scroll_left(&mut self) {
        self.x_position = self.x_position.saturating_sub(10);
    }

    fn set_fold(&mut self, is_fold: bool) {
        self.fold = is_fold;
    }

    fn handle_mouse_events(&mut self, event: MouseEvent) {
        if !self.rect.contains(Position {
            x: event.column,
            y: event.row,
        }) {
            return;
        }

        match event.kind {
            MouseEventKind::ScrollDown => self.scroll_down(),
            MouseEventKind::ScrollUp => self.scroll_up(),
            MouseEventKind::ScrollLeft => self.scroll_left(),
            MouseEventKind::ScrollRight => self.scroll_right(),
            _ => {}
        }
    }
}

fn text_width(text: &Text) -> usize {
    text.lines()
        .into_iter()
        .map(|l| l.cell_width())
        .max()
        .unwrap_or(0) as usize
}

fn text_height(text: &Text) -> usize {
    text.lines().len()
}

impl Component for ExecutionResult {
    fn update(&mut self, action: Action) {
        match action {
            Action::SetResult(result) => self.set_result(result),
            Action::ResultScrollDown => self.scroll_down(),
            Action::ResultScrollUp => self.scroll_up(),
            Action::ScrollRight => self.scroll_right(),
            Action::ScrollLeft => self.scroll_left(),
            Action::ResultPageUp => self.page_up(),
            Action::ResultPageDown => self.page_down(),
            Action::ResultHalfPageDown => self.half_page_down(),
            Action::ResultHalfPageUp => self.half_page_up(),
            Action::SetFold(is_fold) => self.set_fold(is_fold),
            Action::BottomOfPage => self.bottom_of_page(),
            Action::TopOfPage => self.top_of_page(),
            Action::MouseEvent(e) => self.handle_mouse_events(e),
            _ => {}
        }
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.rect = area;

        let text = self.result.clone().unwrap_or(Text::new(""));
        let mut current = text.to_string();
        let mut y_max;
        let mut x_max;
        if self.fold {
            x_max = area.width as usize;
            let folded_text = fold_text(&text, x_max);
            current = folded_text.to_string();
            y_max = text_height(&folded_text);
            if y_max > area.height as usize {
                x_max = (area.width - 1) as usize;
                let folded_text = fold_text(&text, x_max);
                current = folded_text.to_string();
                y_max = text_height(&folded_text);
            }

            self.x_position = 0;
        } else {
            x_max = text_width(&text);
            y_max = text_height(&text);
        }

        let mut body = area;

        let mut y_scrollable = y_max.saturating_sub(body.height as usize);
        let mut x_scrollable = x_max.saturating_sub(body.width as usize);
        let scroll_style = self.config.get_style("scrollbar");

        if y_scrollable > 0 {
            body.width = area.width.saturating_sub(1);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .style(scroll_style)
                .thumb_symbol("║");
            let mut scrollbar_state =
                ScrollbarState::new(y_scrollable).position(self.y_position as usize);
            f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
            if x_max > body.width as usize {
                x_scrollable = x_scrollable.saturating_add(1);
            }
        }

        if x_scrollable > 0 {
            body.height = area.height.saturating_sub(1);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                .symbols(scrollbar::HORIZONTAL)
                .style(scroll_style)
                .thumb_symbol("=");
            let mut scrollbar_state =
                ScrollbarState::new(x_scrollable).position(self.x_position as usize);
            f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
            if y_max > body.height as usize {
                y_scrollable = y_scrollable.saturating_add(1);
            }
        }

        if y_scrollable > 0 {
            body.width = area.width.saturating_sub(1);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .style(scroll_style)
                .thumb_symbol("║");
            let mut scrollbar_state =
                ScrollbarState::new(y_scrollable).position(self.y_position as usize);
            f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
            if x_max > body.width as usize {
                x_scrollable = x_scrollable.saturating_add(1);
            }
        }

        if self.x_position > x_scrollable as u16 {
            self.x_position = x_scrollable as u16;
        }

        if self.y_position > y_scrollable as u16 {
            self.y_position = y_scrollable as u16;
        }

        let current = current.into_text()?;
        let paragraph = Paragraph::new(current).scroll((self.y_position, self.x_position));
        f.render_widget(paragraph, body);

        self.y_max_scroll_size = y_scrollable as u16;
        self.y_area_size = body.height;

        Ok(())
    }
}

fn fold_text(str: &Text, width: usize) -> Text {
    let mut result = Text::new("");
    let mut current = 0;
    let mut previous_style = anstyle::Style::default();
    let unified_str = str.chars.iter().map(|c| c.c).collect::<String>();
    let graphemes = unified_str.graphemes(true);
    let mut cstyles = str.chars.iter().map(|c| c.style);
    for g in graphemes {
        if matches!(g, "\n" | "\r" | "\r\n" | "\u{000B}" | "\u{000C}") {
            current = 0;
            for c in g.chars() {
                let style = cstyles.next().unwrap();
                result.add_char(Char { c, style });
                previous_style = style;
            }
            continue;
        }

        if current == width {
            let char = Char {
                c: '\n',
                style: previous_style,
            };
            result.add_char(char);

            current = 0;
        }

        if current + g.cell_width() as usize > width {
            let char = Char {
                c: '\n',
                style: previous_style,
            };
            result.add_char(char);
            current = 0;
        }

        for c in g.chars() {
            let style = cstyles.next().unwrap();
            result.add_char(Char { c, style });
            previous_style = style;
        }
        current += g.cell_width() as usize;
    }

    result
}

#[cfg(test)]
mod tests {
    use ansi_parser::{AnsiParser as _, Output};

    use super::*;

    fn remove_ansi(text: &str) -> String {
        text.ansi_parse()
            .filter(|o| matches!(o, Output::TextBlock(_)))
            .map(|o| o.to_string())
            .collect::<String>()
    }

    #[test]
    fn test_fold_text() {
        let text = Text::new("hello world");
        let result = fold_text(&text, 5);
        assert_eq!(result.to_string(), "hello\n worl\nd");
    }

    #[test]
    fn test_fold_text_long() {
        let str = r#"use std::{collections::HashMap, time::Duration};
use chrono::{DateTime, Local};
use color_eyre::eyre::Result;
    "#;
        let text = Text::new(str);

        let result = fold_text(&text, 97);

        assert_eq!(result.to_string(), str)
    }

    #[test]
    fn test_fold_text_wide_chars() {
        let text = Text::new("あいうえおかきくけこさしすせそたちつてとなにぬねの");

        let result = fold_text(&text, 10);

        assert_eq!(
            result.to_string(),
            "あいうえお\nかきくけこ\nさしすせそ\nたちつてと\nなにぬねの"
        )
    }

    #[test]
    fn test_fold_text_wide_chars_2() {
        let text = Text::new("iあいうえおかきくけこさしすせそたちつてとなにぬねの");

        let result = fold_text(&text, 10);

        assert_eq!(
            result.to_string(),
            "iあいうえ\nおかきくけ\nこさしすせ\nそたちつて\nとなにぬね\nの"
        )
    }

    #[test]
    fn test_fold_text_does_not_split_grapheme_clusters() {
        let text = Text::new("e\u{301}e\u{301}");

        let result = fold_text(&text, 1);

        assert_eq!(result.to_string(), "e\u{301}\ne\u{301}");
        assert_eq!(
            result.chars.iter().map(|c| c.c).collect::<Vec<_>>(),
            vec!['e', '\u{301}', '\n', 'e', '\u{301}']
        );
    }

    #[test]
    fn test_fold_text_resets_width_after_crlf() {
        let text = Text::new("12\r\n345");

        let result = fold_text(&text, 2);

        assert_eq!(result.to_string(), "12\r\n34\n5");
        assert_eq!(
            result.chars.iter().map(|c| c.c).collect::<Vec<_>>(),
            vec!['1', '2', '\r', '\n', '3', '4', '\n', '5']
        );
    }

    #[test]
    fn test_fold_text_resets_width_after_control_line_breaks() {
        let text = Text::new("12\r34\u{000B}56\u{000C}78");

        let result = fold_text(&text, 2);

        assert_eq!(result.to_string(), "12\r34\u{000B}56\u{000C}78");
    }

    #[test]
    fn test_remove_ansi() {
        let text = "\u{1b}[31mredtextredtext\u{1b}[0m";
        let result = remove_ansi(text);
        assert_eq!(result, "redtextredtext");
    }
}
