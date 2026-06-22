use color_eyre::eyre::Result;
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, tui::Frame};

pub mod clock;
pub mod command;
pub mod execution_result;
pub mod fps;
pub mod help;
pub mod history;
pub mod home;
pub mod interval;
pub mod prompt;
pub mod status;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
/// Implementers of this trait can be registered with the main application loop and will be able to receive events,
/// update state, and be rendered on the screen.
#[allow(unused_variables)]
pub trait Component {
    /// Register an action handler that can send actions for processing if necessary.
    ///
    /// # Arguments
    ///
    /// * `tx` - An unbounded sender that can send actions.
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {}

    /// Update the state of the component based on a received action. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `action` - An action that may modify the state of the component.
    fn update(&mut self, action: Action, area: Rect) {}

    /// Render the component on the screen. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `f` - A frame used for rendering.
    /// * `area` - The area in which the component should be drawn.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()>;
}
