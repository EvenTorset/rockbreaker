use std::{
  collections::HashMap,
  path::PathBuf,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use tokio::sync::oneshot;

use crate::{
  api::{register_item, RegisterResponse},
  state::{AppState, ItemError},
};

/// Explorer starts a separate process for every selected item, all at nearly the same moment. A selection is
/// considered complete once nothing new has arrived for this long.
const QUIET_PERIOD: Duration = Duration::from_millis(400);

/// A selection is closed after this long even if items keep trickling in.
const MAX_AGE: Duration = Duration::from_secs(5);

type Reply = Result<RegisterResponse, ItemError>;

struct Batch {
  paths: Vec<PathBuf>,
  waiters: Vec<oneshot::Sender<Reply>>,
  generation: u64,
  started: Instant,
}

#[derive(Default)]
pub struct SelectionBatcher {
  batches: Mutex<HashMap<bool, Batch>>,
}

impl SelectionBatcher {
  pub async fn submit(self: &Arc<Self>, state: AppState, direct: bool, paths: Vec<PathBuf>) -> Reply {
    let (tx, rx) = oneshot::channel();

    let generation = {
      let mut batches = self.batches.lock().unwrap();
      let batch = batches.entry(direct).or_insert_with(|| Batch {
        paths: Vec::new(),
        waiters: Vec::new(),
        generation: 0,
        started: Instant::now(),
      });
      batch.paths.extend(paths);
      batch.waiters.push(tx);
      batch.generation += 1;
      batch.generation
    };

    let this = self.clone();
    tokio::spawn(async move {
      tokio::time::sleep(QUIET_PERIOD).await;

      let finished = {
        let mut batches = this.batches.lock().unwrap();
        let done = batches
          .get(&direct)
          .is_some_and(|batch| batch.generation == generation || batch.started.elapsed() >= MAX_AGE);
        done.then(|| batches.remove(&direct)).flatten()
      };

      if let Some(batch) = finished {
        let reply = register_item(&state, batch.paths, direct, None);
        for (index, waiter) in batch.waiters.into_iter().enumerate() {
          let _ = waiter.send(reply.clone().map(|response| RegisterResponse { primary: index == 0, ..response }));
        }
      }
    });

    rx.await
      .unwrap_or_else(|_| Err(ItemError::Database("the selection was dropped".to_string())))
  }
}
