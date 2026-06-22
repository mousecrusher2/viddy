use chrono::{DateTime, Local};
use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};

use super::{Component, Frame};
use crate::{action::Action, config::Config};

pub struct Clock {
    config: Config,
    time: Option<DateTime<Local>>,
}

impl Clock {
    pub fn new(config: Config) -> Self {
        Self { config, time: None }
    }
}

impl Component for Clock {
    fn update(&mut self, action: Action, _area: Rect) {
        if let Action::SetClock(datetime) = action {
            self.time = Some(datetime);
        }
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let block = Block::default()
            .title("Time")
            .borders(Borders::ALL)
            .border_style(self.config.get_style("border"))
            .title_style(self.config.get_style("title"));
        let text = self
            .time
            .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default();
        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
        Ok(())
    }
}
