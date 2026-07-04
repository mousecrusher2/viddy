use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::Paragraph};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::action::Action;

#[derive(Default)]
pub struct Prompt {
    command_tx: Option<UnboundedSender<Action>>,
    input: Input,
    is_searching: bool,
    is_inputtig: bool,
}

impl Prompt {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.command_tx = Some(tx);
    }

    #[must_use]
    pub fn cursor_position(&self, area: Rect) -> Option<Position> {
        if !self.is_inputtig {
            return None;
        }

        let cursor_offset =
            u16::try_from(self.input.visual_cursor().saturating_add(1)).unwrap_or(u16::MAX);
        Some(Position::new(area.x.saturating_add(cursor_offset), area.y))
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        self.input
            .handle_event(&crossterm::event::Event::Key(key_event));
        if let Some(tx) = &self.command_tx {
            tx.send(Action::SetSearchQuery(self.input.value().to_string()))
                .expect("action receiver should be alive");
        }
    }

    fn enter_search_mode(&mut self) {
        self.is_inputtig = true;
        self.is_searching = true;
        self.input = Input::default();
        self.command_tx
            .as_ref()
            .expect("action sender should be registered")
            .send(Action::SetSearchQuery(self.input.value().to_string()))
            .expect("action receiver should be alive");
    }

    fn exit_search_mode(&mut self) {
        self.is_inputtig = false;
        self.is_searching = false;
        self.input = Input::default();
        // if let Some(tx) = &self.command_tx {
        //     tx.send(Action::SetSearchQuery(self.input.value().to_string()))?;
        // }
        self.command_tx
            .as_ref()
            .expect("action sender should be registered")
            .send(Action::SetSearchQuery(self.input.value().to_string()))
            .expect("action receiver should be alive");
    }

    fn execute_search(&mut self) {
        self.is_inputtig = false;
        self.is_searching = true;
    }

    pub fn update(&mut self, action: &Action) {
        match *action {
            Action::EnterSearchMode => self.enter_search_mode(),
            Action::KeyEventForPrompt(key_event) => self.handle_key_event(key_event),
            Action::ExecuteSearch => self.execute_search(),
            Action::ExitSearchMode => self.exit_search_mode(),
            _ => {}
        }
    }
}

impl Widget for &Prompt {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let input = if self.is_searching {
            format!("/{}", self.input.value())
        } else {
            String::new()
        };
        let paragraph = Paragraph::new(input);
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent};

    #[test]
    fn cursor_position_is_only_returned_while_inputting() {
        let mut prompt = Prompt {
            is_inputtig: true,
            is_searching: true,
            ..Prompt::default()
        };
        prompt.handle_key_event(KeyEvent::from(KeyCode::Char('a')));

        assert_eq!(
            prompt.cursor_position(Rect::new(2, 3, 10, 1)),
            Some(Position::new(4, 3))
        );

        prompt.is_inputtig = false;
        assert_eq!(prompt.cursor_position(Rect::new(2, 3, 10, 1)), None);
    }
}
