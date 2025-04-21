use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::Weak;

use futures_util::future::BoxFuture;
use parking_lot::Mutex;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct InflightSlots<K, V> {
    inner: Arc<Inner<K, V>>,
}

struct Inner<K, V> {
    slots: Mutex<HashMap<K, Inflight<V>>>,
    work: InflightWork<K, V>,
}

pub type InflightWork<K, V> = Box<dyn Fn(&K) -> BoxFuture<'static, eyre::Result<V>> + Send + Sync>;

pub type Inflight<V> = Weak<broadcast::Sender<Result<V, Arc<eyre::Report>>>>;

impl<K, V> InflightSlots<K, V>
where
    K: Eq + Hash + Clone + Send + 'static,
    V: Clone + Send + 'static,
{
    pub fn new(
        work: impl Fn(&K) -> BoxFuture<'static, eyre::Result<V>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                slots: Mutex::new(HashMap::new()),
                work: Box::new(work),
            }),
        }
    }

    pub async fn query(&self, key: K) -> Result<V, Arc<eyre::Report>> {
        let mut rx = 'make_rx: {
            let mut slots = self.inner.slots.lock();
            if slots.len() > 2048 {
                // time for spring cleaning
                slots.retain(|_, v| v.strong_count() > 0);
            }

            if let Some(inflight) = slots.get(&key).and_then(|v| v.upgrade()) {
                break 'make_rx inflight.subscribe();
            }

            let (tx, rx) = broadcast::channel(1);
            let tx = Arc::new(tx);
            slots.insert(key.clone(), Arc::downgrade(&tx));

            let inner = self.inner.clone();
            let key_clone = key.clone();
            tokio::spawn(async move {
                let res = (inner.work)(&key_clone).await.map_err(Arc::new);
                {
                    let mut slots = inner.slots.lock();
                    slots.remove(&key_clone);
                }
                tx.send(res).ok();
            });
            rx
        };

        rx.recv()
            .await
            .map_err(|_| eyre::eyre!("Inflight task failed"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn test_inflight_deduplication() {
        let counter = Arc::new(AtomicUsize::new(0));

        let slots = InflightSlots::new({
            let counter = counter.clone();
            move |_k: &u32| {
                let counter = counter.clone();
                Box::pin(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Ok(42)
                })
            }
        });

        let key = 1;
        let tasks: Vec<_> = (0..5)
            .map(|_| {
                let slots = slots.clone();
                tokio::spawn(async move { slots.query(key).await.unwrap() })
            })
            .collect();

        let results = futures_util::future::join_all(tasks).await;

        assert_eq!(results.len(), 5);
        for result in results {
            assert_eq!(result.unwrap(), 42);
        }

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
