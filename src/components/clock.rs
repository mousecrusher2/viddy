use chrono::{DateTime, Local};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{action::Action, config::Config};

pub struct Clock {
    config: Config,
    time: Option<DateTime<Local>>,
}

impl Clock {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config, time: None }
    }

    pub fn update(&mut self, action: &Action) {
        if let &Action::SetClock(datetime) = action {
            self.time = Some(datetime);
        }
    }
}

impl Widget for &Clock {
    fn render(self, area: Rect, buf: &mut Buffer) {
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
        paragraph.render(area, buf);
    }
}
