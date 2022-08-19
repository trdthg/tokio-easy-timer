use std::{future::Future, pin::Pin};

use crate::job::{Job, JobId};

mod archived_scheduler;
pub mod bucket;
mod bucket_scheduler;
mod heap_scheduler;
pub mod item;
pub use bucket_scheduler::BucketScheduler;
use chrono::TimeZone;
pub use heap_scheduler::HeapScheduler;

pub type BoxedJob<Tz> = Box<dyn Job<Tz> + Send + 'static>;

// mod private_scheduler {
//     use super::item::ScheduleItem;

//     pub trait Scheduler {
//         fn add_to_schedule(&self, item: ScheduleItem);
//     }
// }

#[async_trait::async_trait]
pub trait Scheduler<Tz: TimeZone> {
    /// Start the timer.
    // fn run(&self) -> Pin<Box<dyn Future<Output = ()>>>;

    /// Start the timer, block the current thread.
    async fn run_pending(&mut self);
    //  {
    //     Box::pin(async {
    //         self.run().await;
    //         std::future::pending::<()>().await;
    //     })
    // }

    /// add a new task to the scheduler, which implements `Job` trait.
    fn add(&mut self, job: BoxedJob<Tz>) -> &mut dyn Scheduler<Tz>;

    fn get(&self, id: &JobId) -> Option<&BoxedJob<Tz>>;

    fn get_mut(&mut self, id: &JobId) -> Option<&mut BoxedJob<Tz>>;

    fn remove(&mut self, id: &JobId);

    fn start(&mut self, id: &JobId) {
        if let Some(job) = self.get_mut(id) {
            job.start();
        }
    }

    fn stop(&mut self, id: &JobId) {
        if let Some(job) = self.get_mut(id) {
            job.stop();
        }
    }
}
