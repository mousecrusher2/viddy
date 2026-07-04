use chrono::Duration as ChronoDuration;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::config::{Config, RuntimeConfig};

pub struct Interval {
    config: Config,
    runtime_config: RuntimeConfig,
}

impl Interval {
    #[must_use]
    pub fn new(config: Config, runtime_config: RuntimeConfig) -> Self {
        Self {
            config,
            runtime_config,
        }
    }

    pub fn increase_interval(&mut self) {
        self.runtime_config.interval +=
            chrono::Duration::milliseconds(self.config.general.interval_step_ms);
    }

    pub fn decrease_interval(&mut self) {
        let min_interval = ChronoDuration::milliseconds(self.config.general.min_interval_ms);
        let step = ChronoDuration::milliseconds(self.config.general.interval_step_ms);
        let new_interval = (self.runtime_config.interval - step).max(min_interval);
        self.runtime_config.interval = new_interval;
    }
}

impl Widget for &Interval {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Every")
            .borders(Borders::ALL)
            .border_style(self.config.get_style("border"))
            .title_style(self.config.get_style("title"));
        let text =
            humantime::format_duration(self.runtime_config.interval.to_std().unwrap_or_default())
                .to_string();
        let paragraph = Paragraph::new(text).block(block);

        paragraph.render(area, buf);
    }
}
