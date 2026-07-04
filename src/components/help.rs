use std::{collections::HashMap, fmt::Write as _};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, StatefulWidget},
};

use crate::{
    action::Action,
    config::{Config, KeyBindings},
    mode::Mode,
};

pub struct Help {
    keybindings: HashMap<(Mode, String), Vec<Vec<KeyEvent>>>,
    y_position: u16,
}

fn keys_str(
    keybindings: &HashMap<(Mode, String), Vec<Vec<KeyEvent>>>,
    mode: Mode,
    action: String,
) -> Vec<Span<'_>> {
    keybindings.get(&(mode, action)).map_or_else(
        || vec![Span::from("None")],
        |keys_list| {
            itertools::Itertools::intersperse(
                keys_list.iter().map(|keys| {
                    let s = keys.iter().fold(String::new(), |mut s, key| {
                        s.push('<');
                        display_key(&mut s, key);
                        s.push('>');
                        s
                    });
                    Span::styled(s, Style::default().fg(Color::Yellow))
                }),
                Span::from(", "),
            )
            .collect()
        },
    )
}

impl Help {
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self {
            keybindings: get_action_keys(&config.keybindings),
            y_position: 0,
        }
    }

    fn scroll_down(&mut self) {
        self.y_position = self.y_position.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.y_position = self.y_position.saturating_sub(1);
    }

    fn page_up(&mut self, area: Rect) {
        self.y_position = self.y_position.saturating_sub(area.height);
    }

    fn page_down(&mut self, area: Rect) {
        self.y_position = self.y_position.saturating_add(area.height);
    }

    fn half_page_up(&mut self, area: Rect) {
        self.y_position = self.y_position.saturating_sub(area.height / 2);
    }

    fn half_page_down(&mut self, area: Rect) {
        self.y_position = self.y_position.saturating_add(area.height / 2);
    }

    fn reset_position(&mut self) {
        self.y_position = 0;
    }

    pub fn update(&mut self, action: &Action, area: Rect) {
        match action {
            Action::ShowHelp => self.reset_position(),
            Action::HelpScrollDown => self.scroll_down(),
            Action::HelpScrollUp => self.scroll_up(),
            Action::HelpPageDown => self.page_down(area),
            Action::HelpPageUp => self.page_up(area),
            Action::HelpHalfPageDown => self.half_page_down(area),
            Action::HelpHalfPageUp => self.half_page_up(area),
            _ => {}
        }
    }
}

fn display_key(s: &mut String, key: &KeyEvent) {
    s.extend(key.modifiers.iter().filter_map(|m| match m {
        KeyModifiers::CONTROL => Some("Ctrl-"),
        KeyModifiers::ALT => Some("Alt-"),
        KeyModifiers::SHIFT => Some("Shift-"),
        _ => None,
    }));

    match key.code {
        KeyCode::Char(' ') => s.push_str("SPACE"),
        KeyCode::Char(c) => s.push(c),
        KeyCode::Enter => s.push_str("Enter"),
        KeyCode::Backspace => s.push_str("Backspace"),
        KeyCode::Left => s.push_str("Left"),
        KeyCode::Right => s.push_str("Right"),
        KeyCode::BackTab => s.push_str("BackTab"),
        KeyCode::Tab => s.push_str("Tab"),
        KeyCode::Home => s.push_str("Home"),
        KeyCode::End => s.push_str("End"),
        KeyCode::Up => s.push_str("Up"),
        KeyCode::Down => s.push_str("Down"),
        KeyCode::PageUp => s.push_str("PageUp"),
        KeyCode::PageDown => s.push_str("PageDown"),
        KeyCode::Delete => s.push_str("Delete"),
        KeyCode::Insert => s.push_str("Insert"),
        KeyCode::F(i) => write!(s, "F{i:?}").unwrap(),
        KeyCode::Null => s.push_str("Null"),
        KeyCode::Esc => s.push_str("Esc"),
        KeyCode::CapsLock => s.push_str("CapsLock"),
        KeyCode::ScrollLock => s.push_str("ScrollLock"),
        KeyCode::NumLock => s.push_str("NumLock"),
        KeyCode::PrintScreen => s.push_str("PrintScreen"),
        KeyCode::Pause => s.push_str("Pause"),
        KeyCode::Menu => s.push_str("Menu"),
        KeyCode::KeypadBegin => s.push_str("KeypadBegin"),
        KeyCode::Media(c) => write!(s, "Media({c:?})").unwrap(),
        KeyCode::Modifier(c) => write!(s, "Modifier({c:?})").unwrap(),
    }
}

pub struct HelpWidget;

impl StatefulWidget for HelpWidget {
    type State = Help;

    fn render(self, area: Rect, buf: &mut Buffer, help: &mut Self::State) {
        let basic_keys = [
            (
                "Toggle time machine mode  ",
                Mode::All,
                Action::SwitchTimemachineMode.to_string(),
            ),
            (
                "Toggle suspend execution  ",
                Mode::All,
                Action::SwitchSuspend.to_string(),
            ),
            (
                "Toggle ring terminal bell ",
                Mode::All,
                Action::SwitchBell.to_string(),
            ),
            (
                "Toggle diff               ",
                Mode::All,
                Action::SwitchDiff.to_string(),
            ),
            (
                "Toggle deletion diff      ",
                Mode::All,
                Action::SwitchDeletionDiff.to_string(),
            ),
            (
                "Toggle header display     ",
                Mode::All,
                Action::SwitchNoTitle.to_string(),
            ),
            (
                "Toggle help view          ",
                Mode::All,
                Action::ShowHelp.to_string(),
            ),
            (
                "Toggle unfold             ",
                Mode::All,
                Action::SwitchFold.to_string(),
            ),
            (
                "Quit Viddy                ",
                Mode::All,
                Action::Quit.to_string(),
            ),
        ];

        let pager_keys = [
            (
                "Search text           ",
                Mode::All,
                Action::EnterSearchMode.to_string(),
            ),
            (
                "Move to next line     ",
                Mode::All,
                Action::ResultScrollDown.to_string(),
            ),
            (
                "Move to previous line ",
                Mode::All,
                Action::ResultScrollUp.to_string(),
            ),
            (
                "Move to right         ",
                Mode::All,
                Action::ScrollRight.to_string(),
            ),
            (
                "Move to left          ",
                Mode::All,
                Action::ScrollLeft.to_string(),
            ),
            (
                "Page down             ",
                Mode::All,
                Action::ResultPageDown.to_string(),
            ),
            (
                "Page up               ",
                Mode::All,
                Action::ResultPageUp.to_string(),
            ),
            (
                "Half page down        ",
                Mode::All,
                Action::ResultHalfPageDown.to_string(),
            ),
            (
                "Half page up          ",
                Mode::All,
                Action::ResultHalfPageUp.to_string(),
            ),
            (
                "Go to top of page     ",
                Mode::All,
                Action::BottomOfPage.to_string(),
            ),
            (
                "Go to bottom of page  ",
                Mode::All,
                Action::TopOfPage.to_string(),
            ),
        ];

        let timemachine_keys = [
            (
                "Go to the past           ",
                Mode::All,
                Action::GoToPast.to_string(),
            ),
            (
                "Back to the future       ",
                Mode::All,
                Action::GoToFuture.to_string(),
            ),
            (
                "Go to more past          ",
                Mode::All,
                Action::GoToMorePast.to_string(),
            ),
            (
                "Back to more future      ",
                Mode::All,
                Action::GoToMoreFuture.to_string(),
            ),
            (
                "Go to oldest position    ",
                Mode::All,
                Action::GoToOldest.to_string(),
            ),
            (
                "Back to current position ",
                Mode::All,
                Action::GoToCurrent.to_string(),
            ),
        ];

        let mut lines = vec![
            Line::from("Press ESC or q to go back"),
            Line::from(""),
            Line::styled(
                " Key Bindings",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::from(vec![
                Span::from("   "),
                Span::styled(
                    "General",
                    Style::default().add_modifier(Modifier::UNDERLINED),
                ),
            ]),
            Line::from(""),
        ];

        lines.extend(basic_keys.map(|(action, mode, key)| {
            let keys_str = keys_str(&help.keybindings, mode, key);
            Line::from(
                [
                    vec![
                        Span::from("   "),
                        Span::styled(action, Style::default().add_modifier(Modifier::BOLD)),
                        Span::from(": "),
                    ],
                    keys_str,
                ]
                .concat(),
            )
        }));

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::from("   "),
            Span::styled("Pager", Style::default().add_modifier(Modifier::UNDERLINED)),
        ]));
        lines.push(Line::from(""));

        lines.extend(pager_keys.map(|(description, mode, action)| {
            let keys_str = keys_str(&help.keybindings, mode, action);
            Line::from(
                [
                    vec![
                        Span::from("   "),
                        Span::styled(description, Style::default().add_modifier(Modifier::BOLD)),
                        Span::from(": "),
                    ],
                    keys_str,
                ]
                .concat(),
            )
        }));

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::from("   "),
            Span::styled(
                "Time machine",
                Style::default().add_modifier(Modifier::UNDERLINED),
            ),
        ]));
        lines.push(Line::from(""));

        lines.extend(timemachine_keys.map(|(action, mode, key)| {
            let keys_str = keys_str(&help.keybindings, mode, key);
            Line::from(
                [
                    vec![
                        Span::from("   "),
                        Span::styled(action, Style::default().add_modifier(Modifier::BOLD)),
                        Span::from(": "),
                    ],
                    keys_str,
                ]
                .concat(),
            )
        }));

        lines.push(Line::from(""));

        let area_height: usize = area.height.into();
        let scrollable_height = lines.len().saturating_sub(area_height);
        let max_y_position = u16::try_from(scrollable_height).unwrap_or(u16::MAX);
        help.y_position = help.y_position.min(max_y_position);

        let paragraph = Paragraph::new(Text::from(lines)).scroll((help.y_position, 0));
        paragraph.render(area, buf);
    }
}

fn get_action_keys(keybindings: &KeyBindings) -> HashMap<(Mode, String), Vec<Vec<KeyEvent>>> {
    keybindings
        .0
        .iter()
        .flat_map(|(mode, bindings)| {
            bindings
                .iter()
                .map(move |(event, action)| ((*mode, action.to_string()), event.clone()))
        })
        .fold(HashMap::new(), |mut action_keys, (key, event)| {
            action_keys.entry(key).or_default().push(event);
            action_keys
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent};
    use std::collections::HashMap;

    #[test]
    fn test_keys_str() {
        let mut keybindings = HashMap::new();
        let mode = Mode::All;
        let action = String::from("action");
        let key_event1 = KeyEvent::from(KeyCode::Char('a'));
        let key_event2 = KeyEvent::from(KeyCode::Char('b'));
        let key_event3 = KeyEvent::from(KeyCode::Char('c'));
        keybindings.insert(
            (mode, action.clone()),
            vec![vec![key_event1], vec![key_event2, key_event3]],
        );

        let result = keys_str(&keybindings, mode, action);

        let expected_output = vec![
            Span::styled("<a>", Style::default().fg(Color::Yellow)),
            Span::from(", "),
            Span::styled("<b><c>", Style::default().fg(Color::Yellow)),
        ];
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_page_navigation_uses_current_area() {
        let mut help = Help {
            keybindings: HashMap::new(),
            y_position: 10,
        };

        help.update(&Action::HelpPageUp, Rect::new(0, 0, 0, 3));
        assert_eq!(help.y_position, 7);

        help.update(&Action::HelpHalfPageDown, Rect::new(0, 0, 0, 6));
        assert_eq!(help.y_position, 10);
    }
}
