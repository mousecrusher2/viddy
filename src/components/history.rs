use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use chrono::{DateTime, Local};
use color_eyre::eyre::{Ok, Result};
use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tui_widget_list::{ListBuilder, ListState, ListView};

use super::{Component, Frame};
use crate::{
    action::Action,
    config::{Config, RuntimeConfig},
    types::ExecutionId,
    widget::history_item::HistoryItem,
};

pub struct History {
    latest_id: Option<ExecutionId>,
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    items: VecDeque<Rc<RefCell<HistoryItem>>>,
    index: HashMap<ExecutionId, Rc<RefCell<HistoryItem>>>,
    state: ListState,
    runtime_config: RuntimeConfig,
    timemachine_mode: bool,
    rect: Rect,
}

impl History {
    pub fn new(config: Config, runtime_config: RuntimeConfig) -> Self {
        let state = ListState::default();
        let index = HashMap::new();
        Self {
            latest_id: None,
            command_tx: None,
            config,
            items: VecDeque::new(),
            state,
            index,
            runtime_config,
            timemachine_mode: false,
            rect: Rect::default(),
        }
    }

    fn update_latest_history_count(&self) {
        if let Some(latest_id) = self.latest_id
            && let Some(record) = self.index.get(&latest_id)
        {
            record.borrow_mut().update_same_count();
        }
    }

    fn insert_history(&mut self, id: ExecutionId, start_time: DateTime<Local>) {
        let item = Rc::new(RefCell::new(HistoryItem::new(
            id,
            start_time,
            self.runtime_config.interval,
            self.config.get_style("timemachine_selector"),
            self.config.get_style("secondary_text"),
        )));
        self.index.insert(id, Rc::clone(&item));
        self.items.push_front(item);
        self.latest_id = Some(id);
        if self.timemachine_mode {
            self.select(self.state.selected.map(|s| s + 1));
        }
    }

    fn update_history_result(&mut self, id: ExecutionId, diff: Option<(u32, u32)>, exit_code: i32) {
        if let Some(item) = self.index.get(&id) {
            item.borrow_mut().update_diff(diff, exit_code);
            if self.timemachine_mode && self.state.selected.is_none() {
                self.select_latest();
            }
        }
    }

    fn set_timemachine_mode(&mut self, timemachine_mode: bool) {
        self.timemachine_mode = timemachine_mode;
        if self.timemachine_mode {
            self.select_latest();
        }
    }

    fn select_latest(&mut self) {
        let index_to_select = self.items.iter().enumerate().find_map(|(i, item)| {
            let item = item.borrow();
            if !item.is_running { Some(i) } else { None }
        });

        self.select(index_to_select)
    }

    fn select(&mut self, index: Option<usize>) {
        if let Some(index) = index
            && let Some(history_item) = self.items.get(index)
        {
            let history_item = history_item.borrow();
            if !history_item.is_running {
                self.state.select(Some(index));

                // if let Some(tx) = &self.command_tx {
                //     tx.send(Action::ShowExecution(history_item.id, history_item.id))?;
                // }
                self.command_tx
                    .as_ref()
                    .expect("action sender should be registered")
                    .send(Action::ShowExecution(history_item.id, history_item.id))
                    .expect("action receiver should be alive");
            }
        }
    }

    fn go_to_past(&mut self) {
        self.select_saturating_add(1)
    }

    fn go_to_more_past(&mut self) {
        self.select_saturating_add(10)
    }

    fn go_to_future(&mut self) {
        self.select_saturating_sub(1)
    }

    fn go_to_more_future(&mut self) {
        self.select_saturating_sub(10)
    }

    fn select_saturating_add(&mut self, n: usize) {
        if !self.timemachine_mode {
            return;
        }

        let selected = self
            .state
            .selected
            .map(|s| s.saturating_add(n).min(self.items.len() - 1));
        if selected.is_none() {
            return;
        }

        self.select(selected)
    }

    fn select_saturating_sub(&mut self, n: usize) {
        if !self.timemachine_mode {
            return;
        }

        if self.state.selected.is_none() {
            return;
        }

        self.select(self.state.selected.map(|s| s.saturating_sub(n)))
    }

    fn go_to_oldest(&mut self) {
        if !self.timemachine_mode {
            return;
        }

        self.select(self.items.len().checked_sub(1))
    }

    fn go_to_current(&mut self) {
        if !self.timemachine_mode {
            return;
        }

        self.select_latest()
    }

    fn handle_mouse_events(&mut self, event: MouseEvent) {
        log::debug!("Mouse event: {:?}", event);
        if !self.rect.contains(Position {
            x: event.column,
            y: event.row,
        }) {
            return;
        }

        match event.kind {
            MouseEventKind::ScrollDown => self.go_to_past(),
            MouseEventKind::ScrollUp => self.go_to_future(),
            _ => (),
        }
    }
}

impl Component for History {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.command_tx = Some(tx);
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::InsertHistory(id, start_time) => self.insert_history(id, start_time),
            Action::UpdateHistoryResult(id, diff, exit_code) => {
                self.update_history_result(id, diff, exit_code)
            }
            Action::UpdateLatestHistoryCount => self.update_latest_history_count(),
            Action::GoToPast => self.go_to_past(),
            Action::GoToFuture => self.go_to_future(),
            Action::SetTimemachineMode(timemachine_mode) => {
                self.set_timemachine_mode(timemachine_mode)
            }
            Action::GoToMoreFuture => self.go_to_more_future(),
            Action::GoToMorePast => self.go_to_more_past(),
            Action::GoToOldest => self.go_to_oldest(),
            Action::GoToCurrent => self.go_to_current(),
            Action::MouseEvent(e) => self.handle_mouse_events(e),
            _ => {}
        }
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.rect = area;

        let block = Block::default()
            .title("History")
            .borders(Borders::ALL)
            .border_style(self.config.get_style("border"))
            .title_style(self.config.get_style("title"));
        let items = self
            .items
            .iter()
            .map(|i| i.borrow().clone())
            .collect::<Vec<_>>();
        let builder = ListBuilder::new(|context| {
            let mut item = items[context.index].clone();
            let height = item.get_height_and_set_context(context);
            (item.clone(), height)
        });
        let list = ListView::new(builder, items.len()).block(block);

        f.render_stateful_widget(list, area, &mut self.state);

        Ok(())
    }
}
