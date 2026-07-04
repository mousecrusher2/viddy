use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::config::{Config, RuntimeConfig};

pub struct Command {
    config: Config,
    runtime_config: RuntimeConfig,
}

impl Command {
    #[must_use]
    pub fn new(config: Config, runtime_config: RuntimeConfig) -> Self {
        Self {
            config,
            runtime_config,
        }
    }
}

impl Widget for &Command {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Command")
            .borders(Borders::ALL)
            .border_style(self.config.get_style("border"))
            .title_style(self.config.get_style("title"));
        let paragraph = Paragraph::new(self.runtime_config.command.join(" ")).block(block);

        paragraph.render(area, buf);
    }
}
