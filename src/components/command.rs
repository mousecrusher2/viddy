use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};

use super::{Component, Frame};
use crate::config::{Config, RuntimeConfig};

pub struct Command {
    config: Config,
    runtime_config: RuntimeConfig,
}

impl Command {
    pub fn new(runtime_config: RuntimeConfig) -> Self {
        Self {
            runtime_config,
            config: Config::new().unwrap(),
        }
    }
}

impl Component for Command {
    fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let block = Block::default()
            .title("Command")
            .borders(Borders::ALL)
            .border_style(self.config.get_style("border"))
            .title_style(self.config.get_style("title"));
        let paragraph = Paragraph::new(self.runtime_config.command.join(" ")).block(block);

        f.render_widget(paragraph, area);
        Ok(())
    }
}
