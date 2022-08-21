use crate::job::Job;

pub mod bucket;
mod heap_scheduler;
pub mod item;
mod job_scheduler;
use chrono::TimeZone;
pub use heap_scheduler::HeapScheduler;
pub use job_scheduler::JobScheduler;

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

    async fn run_pending(&mut self) {
        timer_cacher::init();
        self.run().await;
    }

    /// Start the timer, block the current thread.
    async fn run(&mut self);

    /// add a new task to the scheduler, which implements `Job` trait.
    fn add(&mut self, job: BoxedJob<Tz>) -> &mut dyn Scheduler<Tz>;

    // fn start(&mut self, id: &JobId) {
    //     if let Some(job) = self.get_mut(id) {
    //         job.start();
    //     }
    // }

    // fn stop(&mut self, id: &JobId) {
    //     if let Some(job) = self.get_mut(id) {
    //         job.stop();
    //     }
    // }
}
