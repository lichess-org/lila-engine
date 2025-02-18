use std::{
    array,
    collections::{hash_map::RandomState, HashMap},
    hash::{BuildHasher, Hash},
    sync::Mutex,
    time::Duration,
};

use tokio::time::sleep;

use crate::hub::IsValid;

const NUM_SHARDS: usize = 128;

pub struct Ongoing<S, R> {
    random_state: RandomState,
    shards: [Mutex<HashMap<S, R>>; NUM_SHARDS],
}

impl<S: Hash + Eq, R> Default for Ongoing<S, R> {
    fn default() -> Ongoing<S, R> {
        Ongoing {
            random_state: RandomState::new(),
            shards: array::from_fn(|_| Mutex::new(HashMap::new())),
        }
    }
}

impl<S: Hash + Eq, R> Ongoing<S, R> {
    pub fn add(&self, selector: S, item: R) {
        self.shard(&selector).lock().unwrap().insert(selector, item);
    }

    pub fn remove(&self, selector: &S) -> Option<R> {
        self.shard(selector).lock().unwrap().remove(selector)
    }

    fn shard(&self, selector: &S) -> &Mutex<HashMap<S, R>> {
        &self.shards[self.random_state.hash_one(selector) as usize % NUM_SHARDS]
    }
}

impl<S, R: IsValid> Ongoing<S, R> {
    pub async fn garbage_collect(&self) {
        loop {
            for shard in &self.shards {
                shard.lock().unwrap().retain(|_, item| item.is_valid());
                sleep(Duration::from_secs(7)).await;
            }
        }
    }
}
