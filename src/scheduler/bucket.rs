use std::sync::Arc;

use parking_lot::Mutex;

use super::task::ScheduleItem;

pub trait Buckets {
    fn new() -> Self;
    fn add(&self, item: ScheduleItem);
    // fn pop(&self) -> Option<(u64, Vec<ScheduleItem>)>;
}

#[derive(Debug, Clone)]
pub struct ItemBucket {
    time: Arc<Mutex<u64>>,
    items: Arc<Mutex<Vec<ScheduleItem>>>,
}

impl ItemBucket {
    pub fn new(time: u64) -> Self {
        Self {
            time: Arc::new(Mutex::new(time)),
            items: Arc::new(Mutex::new(vec![])),
        }
    }
    pub fn forward_n(&self, n: u64) {
        *self.time.lock() -= n;
    }

    pub fn get_time(&self) -> u64 {
        let res = *self.time.lock();
        res
    }

    pub fn push(&self, item: ScheduleItem) {
        self.items.lock().push(item);
    }
    pub fn get_items(&self) -> Vec<ScheduleItem> {
        let mut res = self.items.lock();
        let ress = std::mem::take(res.as_mut());
        ress
    }
}

impl Ord for ItemBucket {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.lock().cmp(&other.time.lock())
    }
}

impl PartialEq for ItemBucket {
    fn eq(&self, other: &Self) -> bool {
        *self.time.lock() == *other.time.lock()
    }
}
impl Eq for ItemBucket {}
impl PartialOrd for ItemBucket {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.lock().partial_cmp(&other.time.lock())
    }
}
