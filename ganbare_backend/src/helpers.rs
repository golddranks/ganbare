use std::sync::RwLock;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::borrow::Borrow;
use std::hash::Hash;
use std::fmt::Debug;
use std::time::{Instant, Duration};
use errors::Result;

pub struct Cache<K, V> {
    expires: Duration,
    cache: RwLock<HashMap<K, (Instant, V)>>,
    expiry_queue: RwLock<VecDeque<(Instant, K)>>,
}   

impl<K: Hash + Eq + Clone + Debug, V: Clone> Cache<K, V> {

    pub fn new(expires: Duration) -> Self {
        Cache {
            expires,
            cache: RwLock::new(HashMap::new()),
            expiry_queue: RwLock::new(VecDeque::new()),
        }
    }

    pub fn insert(&self, key: K, value: V) -> Result<()> {
        if let (Ok(mut cache), Ok(mut queue)) = (self.cache.write(), self.expiry_queue.write()) {
            let expires = Instant::now() + self.expires;
            cache.insert(key.clone(), (expires, value));
            queue.push_back((expires, key));
            Ok(())
        } else {
            Err("Poisoned locks?".into())
        }
    }

    pub fn get<Q>(&self, key: &Q)
        -> Result<Option<V>>
        where   K: Borrow<Q>,
                Q: Hash + Eq
    {
        if let Ok(cache) = self.cache.read() {
            if let Some(&(expires, ref val)) = cache.get(key).clone() {
                if Instant::now() > expires {
                    Ok(None) // The map is not cleaned yet, but the value has expired
                } else {
                    Ok(Some(val.clone()))
                }
            } else {
                Ok(None)
            }
        } else {
            Err("Poisoned locks?".into())
        }
    }

    pub fn clean_expired(&self) -> Result<(usize, usize)> {
        if let (Ok(mut cache), Ok(mut queue)) = (self.cache.write(), self.expiry_queue.write()) {
            let mut removed = 0;
            loop {
                if match queue.front() {
                    Some(&(ref expires, _)) if &Instant::now() > expires => true,
                    _ => false,
                } {
                    let (expires, key) = queue.pop_front().unwrap();
                    cache.remove(&key);
                    debug!("Removed {:?} (expired: {:?}, now: {:?}) from cache.", key, expires, Instant::now());
                    removed += 1;
                } else {
                    break;
                }
            }
            Ok((queue.len(), removed))
        } else {
            Err("Poisoned locks?".into())
        }
    }
}
