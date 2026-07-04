use std::time::Instant;

use ratatui::{prelude::*, widgets::Block};

use crate::action::Action;

#[derive(Debug, Clone, PartialEq)]
pub struct FpsCounter {
    app_start_time: Instant,
    app_frames: u32,
    app_fps: f64,

    render_start_time: Instant,
    render_frames: u32,
    render_fps: f64,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            app_start_time: Instant::now(),
            app_frames: 0,
            app_fps: 0.0,
            render_start_time: Instant::now(),
            render_frames: 0,
            render_fps: 0.0,
        }
    }

    fn app_tick(&mut self) {
        self.app_frames += 1;
        let now = Instant::now();
        let elapsed = (now - self.app_start_time).as_secs_f64();
        if elapsed >= 1.0 {
            self.app_fps = f64::from(self.app_frames) / elapsed;
            self.app_start_time = now;
            self.app_frames = 0;
        }
    }

    fn render_tick(&mut self) {
        self.render_frames += 1;
        let now = Instant::now();
        let elapsed = (now - self.render_start_time).as_secs_f64();
        if elapsed >= 1.0 {
            self.render_fps = f64::from(self.render_frames) / elapsed;
            self.render_start_time = now;
            self.render_frames = 0;
        }
    }

    pub fn update(&mut self, action: &Action) {
        match action {
            Action::Tick => self.app_tick(),
            Action::Render => self.render_tick(),
            _ => {}
        }
    }
}

impl Widget for &FpsCounter {
    fn render(self, rect: Rect, buf: &mut Buffer) {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), // first row
                Constraint::Min(0),
            ])
            .split(rect);

        let rect = rects[0];

        let s = format!(
            "{:.2} ticks per sec (app) {:.2} frames per sec (render)",
            self.app_fps, self.render_fps
        );
        let block = Block::default().title(Line::from(s.dim()).right_aligned());
        block.render(rect, buf);
    }
}
