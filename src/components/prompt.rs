use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::{Input, backend::crossterm::EventHandler};

use super::{Component, Frame};
use crate::action::Action;

#[derive(Default)]
pub struct Prompt {
    command_tx: Option<UnboundedSender<Action>>,
    pub input: Input,
    is_searching: bool,
    is_inputtig: bool,
}

impl Prompt {
    pub fn new() -> Self {
        Self::default()
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
}

impl Component for Prompt {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.command_tx = Some(tx);
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::EnterSearchMode => self.enter_search_mode(),
            Action::KeyEventForPrompt(key_event) => self.handle_key_event(key_event),
            Action::ExecuteSearch => self.execute_search(),
            Action::ExitSearchMode => self.exit_search_mode(),
            _ => {}
        }
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if self.is_inputtig {
            f.set_cursor_position(Position::new(
                area.x + self.input.visual_cursor() as u16 + 1,
                area.y,
            ));
        }

        let input = if !self.is_searching {
            String::new()
        } else {
            format!("/{}", self.input.value())
        };
        let paragraph = Paragraph::new(input);
        f.render_widget(paragraph, area);

        Ok(())
    }
}
