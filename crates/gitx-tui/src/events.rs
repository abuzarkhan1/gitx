use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent};
use futures::{FutureExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;

pub enum Event {
    Key(KeyEvent),
    Tick,
    Resize(u16, u16),
}

pub struct EventHandler {
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let _sender = sender.clone();
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);

            loop {
                let interval_delay = interval.tick();
                let event = reader.next().fuse();

                tokio::select! {
                    _ = interval_delay => {
                        if _sender.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                    maybe_event = event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                match evt {
                                    CrosstermEvent::Key(key) if _sender.send(Event::Key(key)).is_err() => {
                                        break;
                                    }
                                    CrosstermEvent::Key(_) => {}
                                    CrosstermEvent::Resize(w, h) if _sender.send(Event::Resize(w, h)).is_err() => {
                                        break;
                                    }
                                    CrosstermEvent::Resize(_, _) => {}
                                    _ => {}
                                }
                            }
                            Some(Err(_)) => break,
                            None => break,
                        }
                    }
                }
            }
        });

        Self { receiver }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }
}
