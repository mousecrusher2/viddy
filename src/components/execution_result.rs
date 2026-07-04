use ansi_to_tui::IntoText;
use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::{
    buffer::CellWidth as _,
    prelude::*,
    text::Text as RatatuiText,
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget},
};
use symbols::scrollbar;
use unicode_segmentation::UnicodeSegmentation as _;

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
    fold: bool,
}

impl ExecutionResult {
    #[must_use]
    pub fn new(config: Config, fold: bool) -> Self {
        Self {
            config,
            result: None,
            fold,
            x_position: 0,
            y_position: 0,
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

    fn scroll_metrics(&self, area: Rect) -> (Rect, u16, u16) {
        let empty = Text::default();
        let text = self.result.as_ref().unwrap_or(&empty);
        let (_, x_max, y_max) = prepared_text(text, self.fold, area);
        scroll_metrics(x_max, y_max, area)
    }

    fn page_up(&mut self, area: Rect) {
        let (body, _, _) = self.scroll_metrics(area);
        self.y_position = self.y_position.saturating_sub(body.height);
    }

    fn page_down(&mut self, area: Rect) {
        let (body, _, _) = self.scroll_metrics(area);
        self.y_position = self.y_position.saturating_add(body.height);
    }

    fn half_page_up(&mut self, area: Rect) {
        let (body, _, _) = self.scroll_metrics(area);
        self.y_position = self.y_position.saturating_sub(body.height / 2);
    }

    fn half_page_down(&mut self, area: Rect) {
        let (body, _, _) = self.scroll_metrics(area);
        self.y_position = self.y_position.saturating_add(body.height / 2);
    }

    fn bottom_of_page(&mut self, area: Rect) {
        let (_, _, y_scrollable) = self.scroll_metrics(area);
        self.y_position = y_scrollable;
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

    fn handle_mouse_events(&mut self, event: MouseEvent, area: Rect) {
        if !area.contains(Position {
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

fn parse_ansi_or_plain_text(
    parsed: std::result::Result<RatatuiText<'static>, ansi_to_tui::Error>,
    plain_text: String,
) -> RatatuiText<'static> {
    parsed.unwrap_or_else(|error| {
        log::warn!("Failed to parse ANSI text for rendering: {error}");
        RatatuiText::from(plain_text)
    })
}

fn ratatui_text(text: &Text) -> RatatuiText<'static> {
    let ansi_text = text.to_string();
    let plain_text = text.plain_text();
    parse_ansi_or_plain_text(ansi_text.into_text(), plain_text)
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

fn scrollable_size(content_size: usize, viewport_size: u16) -> u16 {
    u16::try_from(content_size.saturating_sub(viewport_size.into())).unwrap_or(u16::MAX)
}

fn prepared_text(text: &Text, fold: bool, area: Rect) -> (Text, usize, usize) {
    if fold {
        let mut x_max = area.width as usize;
        let mut folded_text = fold_text(text, x_max);
        let mut y_max = text_height(&folded_text);
        if y_max > area.height.into() {
            x_max = (area.width - 1) as usize;
            folded_text = fold_text(text, x_max);
            y_max = text_height(&folded_text);
        }

        (folded_text, x_max, y_max)
    } else {
        (text.clone(), text_width(text), text_height(text))
    }
}

fn scroll_metrics(x_max: usize, y_max: usize, area: Rect) -> (Rect, u16, u16) {
    let mut body = area;
    let mut y_scrollable = scrollable_size(y_max, body.height);
    let mut x_scrollable = scrollable_size(x_max, body.width);

    if y_scrollable > 0 {
        body.width = area.width.saturating_sub(1);
        if x_max > body.width.into() {
            x_scrollable = x_scrollable.saturating_add(1);
        }
    }

    if x_scrollable > 0 {
        body.height = area.height.saturating_sub(1);
        if y_max > body.height.into() {
            y_scrollable = y_scrollable.saturating_add(1);
        }
    }

    if y_scrollable > 0 {
        body.width = area.width.saturating_sub(1);
        if x_max > body.width.into() {
            x_scrollable = x_scrollable.saturating_add(1);
        }
    }

    (body, x_scrollable, y_scrollable)
}

impl ExecutionResult {
    pub fn update(&mut self, action: &Action, area: Rect) {
        match *action {
            Action::SetResult(ref result) => self.set_result(result.clone()),
            Action::ResultScrollDown => self.scroll_down(),
            Action::ResultScrollUp => self.scroll_up(),
            Action::ScrollRight => self.scroll_right(),
            Action::ScrollLeft => self.scroll_left(),
            Action::ResultPageUp => self.page_up(area),
            Action::ResultPageDown => self.page_down(area),
            Action::ResultHalfPageDown => self.half_page_down(area),
            Action::ResultHalfPageUp => self.half_page_up(area),
            Action::SetFold(is_fold) => self.set_fold(is_fold),
            Action::BottomOfPage => self.bottom_of_page(area),
            Action::TopOfPage => self.top_of_page(),
            Action::MouseEvent(e) => self.handle_mouse_events(e, area),
            _ => {}
        }
    }
}

pub struct ExecutionResultWidget;

impl StatefulWidget for ExecutionResultWidget {
    type State = ExecutionResult;

    fn render(self, area: Rect, buf: &mut Buffer, execution_result: &mut Self::State) {
        let empty = Text::default();
        let text = execution_result.result.as_ref().unwrap_or(&empty);
        let (current, x_max, y_max) = prepared_text(text, execution_result.fold, area);
        if execution_result.fold {
            execution_result.x_position = 0;
        }

        let (body, x_scrollable, y_scrollable) = scroll_metrics(x_max, y_max, area);
        let scroll_style = execution_result.config.get_style("scrollbar");

        if y_scrollable > 0 {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .style(scroll_style)
                .thumb_symbol("║");
            let mut scrollbar_state = ScrollbarState::new(y_scrollable.into())
                .position(execution_result.y_position.into());
            scrollbar.render(area, buf, &mut scrollbar_state);
        }

        if x_scrollable > 0 {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
                .symbols(scrollbar::HORIZONTAL)
                .style(scroll_style)
                .thumb_symbol("=");
            let mut scrollbar_state = ScrollbarState::new(x_scrollable.into())
                .position(execution_result.x_position.into());
            scrollbar.render(area, buf, &mut scrollbar_state);
        }

        if execution_result.x_position > x_scrollable {
            execution_result.x_position = x_scrollable;
        }

        if execution_result.y_position > y_scrollable {
            execution_result.y_position = y_scrollable;
        }

        let current = ratatui_text(&current);
        let paragraph = Paragraph::new(current)
            .scroll((execution_result.y_position, execution_result.x_position));
        paragraph.render(body, buf);
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
        let str = r"use std::{collections::HashMap, time::Duration};
use chrono::{DateTime, Local};
use color_eyre::eyre::Result;
    ";
        let text = Text::new(str);

        let result = fold_text(&text, 97);

        assert_eq!(result.to_string(), str);
    }

    #[test]
    fn test_fold_text_wide_chars() {
        let text = Text::new("あいうえおかきくけこさしすせそたちつてとなにぬねの");

        let result = fold_text(&text, 10);

        assert_eq!(
            result.to_string(),
            "あいうえお\nかきくけこ\nさしすせそ\nたちつてと\nなにぬねの"
        );
    }

    #[test]
    fn test_fold_text_wide_chars_2() {
        let text = Text::new("iあいうえおかきくけこさしすせそたちつてとなにぬねの");

        let result = fold_text(&text, 10);

        assert_eq!(
            result.to_string(),
            "iあいうえ\nおかきくけ\nこさしすせ\nそたちつて\nとなにぬね\nの"
        );
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

    #[test]
    fn test_parse_ansi_fallback_uses_plain_text() {
        let result = parse_ansi_or_plain_text(
            Err(ansi_to_tui::Error::NomError("boom".to_string())),
            "plain text".to_string(),
        );

        assert_eq!(result.lines.len(), 1);
        assert_eq!(result.lines[0].spans.len(), 1);
        assert_eq!(result.lines[0].spans[0].content.as_ref(), "plain text");
    }
}
