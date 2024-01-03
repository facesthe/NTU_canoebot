//! This module holds on to spawned threads and monitors their result.

use std::{error::Error, sync::Arc};

use lazy_static::lazy_static;
use tokio::{sync::Mutex, task::JoinHandle};

lazy_static! {
    /// Holds on to `JoinHandle` and logs any errors
    pub static ref THREAD_WATCH: ThreadWatch = {
        ThreadWatch {
            inner: Arc::new(Mutex::new(Inner {
                handles: Vec::new(),
            })),
        }
    };
}

/// Error trait object
pub type DynError = Box<dyn Error + Send + Sync>;

/// Result trait object
pub type DynResult = Result<(), DynError>;

/// Enqueues tasks during their execution and log the contents of any errors.
pub struct ThreadWatch {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    handles: Vec<JoinHandle<DynResult>>,
}

impl ThreadWatch {
    /// Add a JoinHandle to be watched.
    ///
    /// The current pool of handles are checked for their completion and dropped if complete.
    ///
    /// Because of this, handles that return errors will be logged only on subsequent invocations.
    pub async fn add(&self, thread: JoinHandle<DynResult>) {
        let mut lock = self.inner.lock().await;
        lock.handles.push(thread);

        let indices = lock
            .handles
            .iter()
            .enumerate()
            .filter_map(|(idx, handle)| match handle.is_finished() {
                true => Some(idx),
                false => None,
            })
            .rev()
            .collect::<Vec<_>>();

        for idx in indices {
            let h = lock.handles.swap_remove(idx);
            match h.await {
                Ok(res) => match res {
                    Ok(_) => (),
                    Err(e) => log::error!("Thread error: {} - {:?}", e, e.source()),
                },
                Err(e) => log::error!("Join error: {}", e),
            }
        }
    }
}
