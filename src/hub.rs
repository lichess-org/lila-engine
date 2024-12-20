use std::{
    array,
    collections::{hash_map::RandomState, HashMap, VecDeque},
    hash::{BuildHasher, Hash},
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{sync::Notify, time::sleep};

const NUM_SHARDS: usize = 64;

const MAX_ITEMS: usize = 1024;

pub trait IsValid {
    fn is_valid(&self) -> bool;
}

pub struct Hub<S, R> {
    random_state: RandomState,
    shards: [Mutex<Shard<S, R>>; NUM_SHARDS],
}

impl<S: Hash + Eq, R: IsValid> Default for Hub<S, R> {
    fn default() -> Hub<S, R> {
        Hub {
            random_state: RandomState::new(),
            shards: array::from_fn(|_| Mutex::new(Shard::new())),
        }
    }
}

impl<S: Hash + Eq + Clone, R: IsValid> Hub<S, R> {
    pub fn submit(&self, selector: S, data: R) {
        let shard = self.shard(&selector);
        shard.lock().unwrap().submit(selector, data);
    }

    pub async fn acquire(&self, selector: S) -> R {
        let shard = self.shard(&selector);
        loop {
            let res = shard.lock().unwrap().acquire(selector.clone());
            match res {
                Ok(item) => return item,
                Err(signal) => signal.notified().await,
            }
        }
    }

    fn shard(&self, selector: &S) -> &Mutex<Shard<S, R>> {
        &self.shards[self.random_state.hash_one(selector) as usize % NUM_SHARDS]
    }
}

impl<S, R: IsValid> Hub<S, R> {
    pub async fn garbage_collect(&self) {
        loop {
            for shard in &self.shards {
                shard.lock().unwrap().garbage_collect();
                sleep(Duration::from_secs(13)).await;
            }
        }
    }
}

struct Shard<S, R> {
    map: HashMap<S, Queue<R>>,
}

impl<S: Eq + Hash, R: IsValid> Shard<S, R> {
    fn new() -> Shard<S, R> {
        Shard {
            map: HashMap::new(),
        }
    }

    fn submit(&mut self, selector: S, data: R) {
        let entry = self.map.entry(selector).or_default();
        if entry.inner.len() < MAX_ITEMS {
            entry.inner.push_back(data);
            entry.signal.notify_one();
        }
    }

    fn acquire(&mut self, selector: S) -> Result<R, Arc<Notify>> {
        let entry = self.map.entry(selector).or_default();
        loop {
            match entry.inner.pop_front() {
                Some(item) if item.is_valid() => return Ok(item),
                Some(_) => continue,
                None => return Err(Arc::clone(&entry.signal)),
            }
        }
    }
}

impl<S, R: IsValid> Shard<S, R> {
    fn garbage_collect(&mut self) {
        self.map.retain(|_, queue| {
            queue.inner.retain(|item| item.is_valid());
            !queue.inner.is_empty()
        });
    }
}

struct Queue<R> {
    signal: Arc<Notify>,
    inner: VecDeque<R>,
}

impl<R> Default for Queue<R> {
    fn default() -> Queue<R> {
        Queue {
            signal: Arc::new(Notify::new()),
            inner: VecDeque::new(),
        }
    }
}
