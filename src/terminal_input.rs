//! Terminal events handler

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event;

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(crossterm::event::KeyEvent),
    /// Mouse click/scroll.
    #[allow(dead_code)]
    Mouse(crossterm::event::MouseEvent),
    /// Terminal resize.
    #[allow(dead_code)]
    Resize(u16, u16),
}

/// Terminal event source.
#[derive(Debug)]
pub struct Reader {
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
}

impl Reader {
    /// Constructs a new instance of [`EventHandler`].
    ///
    /// # Panics
    ///
    /// Will panic on various I/O errors.
    #[must_use]
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler = {
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    if event::poll(timeout).expect("unable to poll for event") {
                        match event::read().expect("unable to read event") {
                            crossterm::event::Event::Key(e) => {
                                if e.kind == event::KeyEventKind::Press {
                                    sender.send(Event::Key(e))
                                } else {
                                    // ignore KeyEventKind::Release on
                                    // windows
                                    Ok(())
                                }
                            }
                            crossterm::event::Event::Mouse(e) => sender.send(Event::Mouse(e)),
                            crossterm::event::Event::Resize(w, h) => {
                                sender.send(Event::Resize(w, h))
                            }
                            _ => unimplemented!(),
                        }
                        .expect("failed to send terminal event");
                    }

                    if last_tick.elapsed() >= tick_rate {
                        sender.send(Event::Tick).expect("failed to send tick event");
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Self { receiver, handler }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if there is no data
    /// available and it's possible for more data to be sent.
    pub fn next(&self) -> Event {
        self.receiver.recv().unwrap()
    }
}
