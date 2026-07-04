use ratatui::{prelude::*, widgets::Paragraph};

use crate::{
    action::{Action, DiffMode},
    config::Config,
};

pub struct Status {
    config: Config,

    is_fold: bool,
    diff_mode: Option<DiffMode>,
    is_suspend: bool,
    is_bell: bool,
    read_only: bool,
}

impl Status {
    #[must_use]
    pub fn new(
        config: Config,
        is_fold: bool,
        diff_mode: Option<DiffMode>,
        is_bell: bool,
        read_only: bool,
    ) -> Self {
        Self {
            config,
            is_fold,
            diff_mode,
            is_suspend: false,
            is_bell,
            read_only,
        }
    }

    pub fn update(&mut self, action: &Action) {
        match *action {
            Action::SetFold(is_fold) => self.is_fold = is_fold,
            Action::SetDiff(diff_mode) => self.diff_mode = diff_mode,
            Action::SetBell(is_bell) => self.is_bell = is_bell,
            Action::SetSuspend(is_suspend) => self.is_suspend = is_suspend,
            _ => {}
        }
    }
}

impl Widget for &Status {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let enabled_style = Style::default().fg(Color::White).bold();
        let disabled_style = self.config.get_style("secondary_text");

        let mut status = vec![Span::styled(
            "[F]old",
            if self.is_fold {
                enabled_style
            } else {
                disabled_style
            },
        )];
        if self.diff_mode.is_some() {
            status.push(Span::styled(" [D]iff", enabled_style));
            status.push(match self.diff_mode {
                Some(DiffMode::Add) => Span::styled("+", Style::new().fg(Color::Green).bold()),
                Some(DiffMode::Delete) => Span::styled("-", Style::new().fg(Color::Red).bold()),
                _ => Span::raw(" "),
            });
        } else {
            status.push(Span::styled(" [D]iff±", disabled_style));
        }

        if self.read_only {
            status.push(Span::styled(
                " Read-only",
                self.config.get_style("readonly"),
            ));
        } else {
            status.push(Span::styled(
                " [S]uspend",
                if self.is_suspend {
                    enabled_style
                } else {
                    disabled_style
                },
            ));
            status.push(Span::styled(
                " [B]ell",
                if self.is_bell {
                    enabled_style
                } else {
                    disabled_style
                },
            ));
        }

        let line = Line::raw("").spans(status);
        let paragraph = Paragraph::new(line).alignment(Alignment::Right);
        paragraph.render(area, buf);
    }
}
