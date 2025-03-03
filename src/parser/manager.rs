use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};

use super::super::builder::{AST, span::FileStorage};

#[derive(Debug)]
pub struct ParseManager {
    active_tasks: AtomicUsize,
    done_tx: Mutex<Option<oneshot::Sender<()>>>,
    pub file_storage: Mutex<FileStorage<AST>>,
}

impl ParseManager {
    pub(super) fn new() -> (Arc<Self>, oneshot::Receiver<()>) {
        let (tx, done_rx) = oneshot::channel();
        (
            Arc::new(ParseManager {
                active_tasks: AtomicUsize::new(0),
                done_tx: Mutex::new(Some(tx)),
                file_storage: Mutex::new(FileStorage::default()),
            }),
            done_rx,
        )
    }

    /// start a new async parse task
    pub fn spawn_parse<F>(self: &Arc<Self>, fut: F)
    where
        F: core::future::Future<Output = ()> + Send + 'static,
    {
        // 1. task num +1
        self.active_tasks.fetch_add(1, Ordering::SeqCst);

        // 2. clone Arc<ParseManager>, move it into async task
        let manager_clone = self.clone();

        // 3. start task
        tokio::spawn(async move {
            // waiting for parser done
            fut.await;
            // task_done
            manager_clone.task_done().await;
        });
    }
    pub(super) async fn wait(self: &Arc<Self>, done_rx: oneshot::Receiver<()>) {
        if 0 != self.active_tasks.load(Ordering::SeqCst) {
            let result = done_rx.await;
            assert!(
                result.is_ok(),
                "The single task should eventually trigger done signal"
            );
        }
    }
    /// internal, record one taks is done
    async fn task_done(&self) {
        let remaining = self.active_tasks.fetch_sub(1, Ordering::SeqCst) - 1;
        if remaining == 0 {
            if let Some(tx) = self.done_tx.lock().await.take() {
                _ = tx.send(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, sleep};

    #[tokio::test]
    async fn test_parse_manager_multiple_tasks() {
        let (manager, done_rx) = ParseManager::new();

        manager.spawn_parse(async {
            for i in 0..3 {
                sleep(Duration::from_millis(50)).await;
                println!("task1-{i}");
            }
            println!("done"); // do something...
        });

        manager.spawn_parse(async {
            for i in 0..5 {
                sleep(Duration::from_millis(30)).await;
                println!("task2-{i}");
            }
            println!("done");
            // do something...
        });

        manager.wait(done_rx).await;
    }

    #[tokio::test]
    async fn test_parse_manager_no_tasks() {
        let (manager, done_rx) = ParseManager::new();
        manager.wait(done_rx).await;
    }

    #[tokio::test]
    async fn test_parse_manager_one_task() {
        let (manager, done_rx) = ParseManager::new();

        manager.spawn_parse(async {
            sleep(Duration::from_millis(10)).await;
            // do parse
        });
        manager.wait(done_rx).await;
    }
}
