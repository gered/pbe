use std::path::Path;
use std::time::Duration;

use async_watcher::{notify::RecursiveMode, AsyncDebouncer, DebouncedEvent};
use tokio::sync::mpsc::channel;

#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
	#[error("Async Debounce Watcher error")]
	AsyncWatcherError(#[from] async_watcher::error::Error),

	#[error("Notify error")]
	NotifyError(#[from] notify::Error),
}

const CHANNEL_BUFFER_SIZE: usize = 100;
const DEBOUNCE_TIME: Duration = Duration::from_secs(1);

pub type DeboundedEventResult = Result<Vec<DebouncedEvent>, Vec<notify::Error>>;

pub trait WatchEventHandler: Send + 'static {
	fn handle_event(&mut self, event: DeboundedEventResult);
}

impl<F> WatchEventHandler for F
where
	F: FnMut(DeboundedEventResult) + Send + 'static,
{
	fn handle_event(&mut self, event: DeboundedEventResult) {
		(self)(event);
	}
}

pub async fn debounce_watch<P, T>(paths: &[P], mut handler: T) -> Result<(), WatcherError>
where
	P: AsRef<Path>,
	T: WatchEventHandler,
{
	log::debug!("Setting up debounced watcher");

	let (tx, mut rx) = channel(CHANNEL_BUFFER_SIZE);

	let mut debouncer = AsyncDebouncer::new(DEBOUNCE_TIME, Some(DEBOUNCE_TIME), tx).await?;

	for path in paths.iter() {
		log::debug!("Watching path {:?}", path.as_ref());
		debouncer.watcher().watch(path.as_ref(), RecursiveMode::Recursive)?;
	}

	while let Some(event) = rx.recv().await {
		handler.handle_event(event)
	}

	Ok(())
}
