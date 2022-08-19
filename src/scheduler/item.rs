use std::sync::{Arc, Mutex};

use crate::job::JobId;

pub trait Item {
    fn get_time(&self) -> u64;
    fn get_id(&self) -> JobId;
}

#[derive(Debug)]
pub struct ScheduleItem {
    pub id: JobId,
    pub time: u64,
}

impl Item for ScheduleItem {
    fn get_id(&self) -> JobId {
        self.id
    }
    fn get_time(&self) -> u64 {
        self.time
    }
}

impl Ord for ScheduleItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

impl PartialEq for ScheduleItem {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl Eq for ScheduleItem {}
impl PartialOrd for ScheduleItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}
