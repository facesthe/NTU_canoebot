//! This module holds on to spawned threads and monitors their result.

use std::{error::Error, sync::Arc, time::Duration};

use futures::Future;
use lazy_static::lazy_static;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

/// The default amount of time to wait for a future to execute to completion
const DEFAULT_DURATION: Duration = Duration::from_secs(15);

lazy_static! {
    /// Holds on to `JoinHandle` and logs any errors
    pub static ref THREAD_WATCH: ThreadWatch = ThreadWatch::new();
}

/// Error trait object
pub type DynError = Box<dyn Error + Send + Sync>;

/// Result trait object
pub type DynResult = Result<(), DynError>;

/// Enqueues tasks during their execution and log the contents of any errors.
pub struct ThreadWatch {
    inner: Arc<Mutex<Inner>>,
    task_send_chan: mpsc::UnboundedSender<()>,
}

struct Inner {
    handles: Vec<JoinHandle<DynResult>>,
}

impl ThreadWatch {
    /// Create a new thread watch instance.
    ///
    /// This also spawns all necessary tasks.
    pub fn new() -> Self {
        let (t_send, t_recv) = mpsc::unbounded_channel();

        let inner = Arc::new(Mutex::new(Inner {
            handles: Default::default(),
        }));

        let s = Self {
            inner: inner.clone(),
            task_send_chan: t_send.clone(),
        };

        // spawn the prune task
        tokio::spawn(Self::run_pruner(inner, t_recv));

        s
    }

    /// This is a direct replacement for [tokio::spawn].
    ///
    /// Only futures that returnn a [DynResult] are supported.
    pub async fn spawn<T>(&self, future: T)
    where
        T: Future + Send + 'static,
        T: Future<Output = DynResult>,
    {
        let handle = tokio::spawn(future);
        self.push(handle, DEFAULT_DURATION).await;
    }

    /// Pushes a joinhandle to the thread queue.
    /// Runs a prune of the thread queue after a specified duration.
    pub async fn push(&self, thread: JoinHandle<DynResult>, prune_delay: Duration) {
        let mut lock = self.inner.lock().await;
        lock.handles.push(thread);
        drop(lock);

        let sender_clone = self.task_send_chan.clone();
        tokio::spawn(async move {
            tokio::time::sleep(prune_delay).await;
            sender_clone
                .send(())
                .expect("unable to send to pruner task");
        });
    }

    /// Runs the pruner. Spawn this function as a new task.
    async fn run_pruner(inner_ref: Arc<Mutex<Inner>>, mut channel: mpsc::UnboundedReceiver<()>) {
        // ..
        while let Some(_) = channel.recv().await {
            let mut lock = inner_ref.lock().await;

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

            // let count = indices.len();

            for idx in indices {
                let h = lock.handles.swap_remove(idx);
                match h.await {
                    Ok(res) => match res {
                        Ok(_) => (),
                        Err(e) => log::error!("Thread error: {} \nCaused by:? {:?}", e, e.source()),
                    },
                    Err(e) => log::error!("Join error: {}", e),
                }
            }

            drop(lock);
        }
    }
}
