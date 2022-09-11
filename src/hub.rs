use std::collections::VecDeque;
use std::sync::Mutex;
use std::collections::HashMap;
use tokio::sync::Notify;
use std::hash::Hash;
use std::sync::Arc;

const MAX_ITEMS: usize = 1024;

pub struct Hub<K, V> {
    shards: [Mutex<Shard<K, V>>; 8],
}

impl<K, V> Hub<K, V> {
    pub fn submit(selector: K, data: V) {

    }

    pub async fn acquire(selector: K) -> V {
        todo!()
    }
}

struct Shard<K, V> {
    map: HashMap<K, Queue<V>>,
}

impl<K: Eq + Hash, V> Shard<K, V> {
    fn submit(&mut self, selector: K, data: V) {
        let entry = self.map.entry(selector).or_default();
        if entry.inner.len() < MAX_ITEMS {
            entry.inner.push_back(data);
            entry.signal.notify_one();
        }
    }

    fn acquire(&mut self, selector: K) -> Result<V, Arc<Notify>> {
        let entry = self.map.entry(selector).or_default();
        entry.inner.pop_front().ok_or_else(|| Arc::clone(&entry.signal))
    }
}

struct Queue<V> {
    signal: Arc<Notify>,
    inner: VecDeque<V>,
}

impl<V> Default for Queue<V> {
    fn default() -> Queue<V> {
        Queue {
            signal: Arc::new(Notify::new()),
            inner: VecDeque::new(),
        }
    }
}
