use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::{
    cursor,
    event::{
        DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEvent, KeyEventKind,
        MouseEvent,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::{backend::CrosstermBackend as Backend, layout::Rect};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

pub type IO = std::io::Stdout;
pub type Frame<'a> = ratatui::Frame<'a>;

pub enum Event {
    Tick,
    Render,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct Tui {
    terminal: ratatui::Terminal<Backend<IO>>,
    task: JoinHandle<()>,
    cancellation_token: CancellationToken,
    event_rx: UnboundedReceiver<Event>,
    event_tx: UnboundedSender<Event>,
    frame_rate: f64,
    tick_rate: f64,
    mouse: bool,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let tick_rate = 4.0;
        let frame_rate = 60.0;
        let terminal = ratatui::Terminal::new(Backend::new(std::io::stdout()))?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let task = tokio::spawn(async {});
        let mouse = false;
        Ok(Self {
            terminal,
            task,
            cancellation_token,
            event_rx,
            event_tx,
            frame_rate,
            tick_rate,
            mouse,
        })
    }

    #[must_use]
    pub fn tick_rate(mut self, tick_rate: f64) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    #[must_use]
    pub fn frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    #[must_use]
    pub fn mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }

    pub fn start(&mut self) {
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);
        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let cancellation_token = self.cancellation_token.clone();
        let event_tx = self.event_tx.clone();
        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);
            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    () = cancellation_token.cancelled() => {
                        break;
                    }
                    maybe_event = crossterm_event => {
                        if let Some(Ok(evt)) = maybe_event {
                            match evt {
                                CrosstermEvent::Key(key) => {
                                    if key.kind == KeyEventKind::Press {
                                        event_tx.send(Event::Key(key)).unwrap();
                                    }
                                },
                                CrosstermEvent::Mouse(mouse) => {
                                    event_tx.send(Event::Mouse(mouse)).unwrap();
                                },
                                CrosstermEvent::Resize(x, y) => {
                                    event_tx.send(Event::Resize(x, y)).unwrap();
                                },
                                CrosstermEvent::FocusGained |CrosstermEvent::FocusLost | CrosstermEvent::Paste(_) => {},
                            }
                        }
                    },
                    _ = tick_delay => {
                        event_tx.send(Event::Tick).unwrap();
                    },
                    _ = render_delay => {
                        event_tx.send(Event::Render).unwrap();
                    },
                }
            }
        });
    }

    pub fn draw(&mut self, render_callback: impl FnOnce(&mut Frame<'_>)) -> Result<()> {
        self.terminal.draw(render_callback)?;
        Ok(())
    }

    pub fn resize(&mut self, area: Rect) -> Result<()> {
        self.terminal.resize(area)?;
        Ok(())
    }

    pub fn size(&self) -> Result<Rect> {
        Ok(self.terminal.size()?.into())
    }

    pub fn stop(&self) -> Result<()> {
        self.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                log::error!("Failed to abort task in 100 milliseconds for unknown reason");
                break;
            }
        }
        Ok(())
    }

    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            self.terminal.backend_mut(),
            EnterAlternateScreen,
            cursor::Hide
        )?;
        if self.mouse {
            crossterm::execute!(self.terminal.backend_mut(), EnableMouseCapture)?;
        }
        self.start();
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        self.stop()?;
        self.terminal.flush()?;
        if self.mouse {
            crossterm::execute!(self.terminal.backend_mut(), DisableMouseCapture)?;
        }
        crossterm::execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            cursor::Show
        )?;
        if crossterm::terminal::is_raw_mode_enabled()? {
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub fn suspend(&mut self) -> Result<()> {
        self.exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        Ok(())
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}
