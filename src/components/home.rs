use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::Block};
use tokio::sync::mpsc::UnboundedSender;

use super::{
    Component, Frame, clock::Clock, command::Command, execution_result::ExecutionResult,
    help::Help, history::History, interval::Interval, prompt::Prompt, status::Status,
};
use crate::{
    action::{Action, DiffMode},
    config::{Config, RuntimeConfig},
    mode::Mode,
};

pub struct Home {
    config: Config,
    is_no_title: bool,

    mode: Mode,
    command_component: Command,
    interval_component: Interval,
    clock_component: Clock,
    execution_result_component: ExecutionResult,
    history_component: History,
    prompt_component: Prompt,
    status_component: Status,
    help_component: Help,
    timemachine_mode: bool,
}

impl Home {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        config: &Config,
        runtime_config: RuntimeConfig,
        is_fold: bool,
        diff_mode: Option<DiffMode>,
        is_bell: bool,
        is_no_title: bool,
        read_only: bool,
        timemachine_mode: bool,
    ) -> Self {
        Self {
            config: config.clone(),
            is_no_title,
            mode: Mode::default(),
            command_component: Command::new(config.clone(), runtime_config.clone()),
            interval_component: Interval::new(config.clone(), runtime_config.clone()),
            clock_component: Clock::new(config.clone()),
            execution_result_component: ExecutionResult::new(config.clone(), is_fold),
            history_component: History::new(config.clone(), runtime_config),
            prompt_component: Prompt::new(),
            status_component: Status::new(config.clone(), is_fold, diff_mode, is_bell, read_only),
            help_component: Help::new(config),
            timemachine_mode,
        }
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn set_timemachine_mode(&mut self, timemachine_mode: bool) {
        self.timemachine_mode = timemachine_mode;
    }

    fn child_areas(&self, area: Rect) -> (Rect, Rect, Rect, Rect, Rect, Rect, Rect) {
        let header_length = if self.is_no_title { 0 } else { 3 };
        let [header, middle, footer] = Layout::vertical([
            Constraint::Length(header_length),
            Constraint::Fill(100),
            Constraint::Length(1),
        ])
        .areas(area);

        let [interval, command, clock] = Layout::horizontal([
            Constraint::Length(10),
            Constraint::Fill(100),
            Constraint::Length(21),
        ])
        .areas(header);

        let [execution_result, history] = if self.timemachine_mode {
            Layout::horizontal([Constraint::Fill(100), Constraint::Length(21)]).areas(middle)
        } else {
            [middle, Rect::default()]
        };

        let [prompt, status] =
            Layout::horizontal([Constraint::Fill(100), Constraint::Length(32)]).areas(footer);

        (
            command,
            interval,
            clock,
            execution_result,
            history,
            prompt,
            status,
        )
    }
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.command_component.register_action_handler(tx.clone());
        self.interval_component.register_action_handler(tx.clone());
        self.clock_component.register_action_handler(tx.clone());
        self.execution_result_component
            .register_action_handler(tx.clone());
        self.history_component.register_action_handler(tx.clone());
        self.prompt_component.register_action_handler(tx.clone());
        self.status_component.register_action_handler(tx.clone());
        self.help_component.register_action_handler(tx);
    }

    fn update(&mut self, action: &Action, area: Rect) {
        match *action {
            Action::SetMode(mode) => self.set_mode(mode),
            Action::SetTimemachineMode(timemachine_mode) => {
                self.set_timemachine_mode(timemachine_mode);
            }
            Action::IncreaseInterval => {
                self.interval_component.increase_interval();
            }
            Action::DecreaseInterval => {
                self.interval_component.decrease_interval();
            }
            Action::SetNoTitle(is_no_title) => self.is_no_title = is_no_title,
            _ => {}
        }

        let default_area = Rect::default();
        let (command, interval, clock, execution_result, history, prompt, status) =
            if self.mode == Mode::Help {
                (
                    default_area,
                    default_area,
                    default_area,
                    default_area,
                    default_area,
                    default_area,
                    default_area,
                )
            } else {
                self.child_areas(area)
            };
        let help = if self.mode == Mode::Help {
            area
        } else {
            default_area
        };

        self.clock_component.update(action, clock);
        self.command_component.update(action, command);
        self.interval_component.update(action, interval);
        self.execution_result_component
            .update(action, execution_result);
        self.history_component.update(action, history);
        self.prompt_component.update(action, prompt);
        self.status_component.update(action, status);
        self.help_component.update(action, help);
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        f.render_widget(
            Block::new().style(self.config.get_style("background")),
            area,
        );

        if self.mode == Mode::Help {
            self.help_component.draw(f, area)?;

            return Ok(());
        }

        let (command, interval, clock, execution_result, history, prompt, status) =
            self.child_areas(area);

        self.command_component.draw(f, command)?;
        self.interval_component.draw(f, interval)?;
        self.clock_component.draw(f, clock)?;

        if self.timemachine_mode {
            self.history_component.draw(f, history)?;
            self.execution_result_component.draw(f, execution_result)?;
        } else {
            self.execution_result_component.draw(f, execution_result)?;
        }

        self.prompt_component.draw(f, prompt)?;
        self.status_component.draw(f, status)?;

        Ok(())
    }
}
