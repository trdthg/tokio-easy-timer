use crate::job::{Job, SyncJobBuilder};

pub mod bucket;
mod exper_wheel;
mod heap_scheduler;
pub mod item;
mod job_scheduler;
use chrono::TimeZone;
pub use heap_scheduler::HeapScheduler;
pub use job_scheduler::JobScheduler;
mod timing_wheel_scheduler;
pub use timing_wheel_scheduler::TimingWheelScheduler;

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

    fn get_tz(&self) -> Tz;

    async fn run_pending(&mut self) {
        timer_cacher::init();
        self.run().await;
    }

    /// Start the timer, block the current thread.
    async fn run(&mut self);

    /// add a new task to the scheduler, which implements `Job` trait.
    fn add_job(&mut self, job: BoxedJob<Tz>) -> &mut dyn Scheduler<Tz>;

    fn add(&mut self, jobs: Vec<BoxedJob<Tz>>) -> &mut dyn Scheduler<Tz>
    where
        Self: Sized,
    {
        for job in jobs {
            self.add_job(job);
        }
        self
    }

    fn add_syncbuilder<Args, F>(
        &mut self,
        jobs: Vec<SyncJobBuilder<Args, F>>,
    ) -> &mut dyn Scheduler<Tz>
    where
        Self: Sized,
        F: crate::job::SyncHandler<Args> + Send + 'static + Copy,
        Args: Send + 'static + Clone,
        Tz: TimeZone + Send + Copy + 'static,
        <Tz as TimeZone>::Offset: Send,
    {
        for job in jobs {
            let job = job.with_tz(self.get_tz());
            self.add_job(Box::new(job));
        }
        self
    }

    fn add_asyncbuilder<Args, F>(
        &mut self,
        jobs: Vec<crate::job::AsyncJobBuilder<Args, F>>,
    ) -> &mut dyn Scheduler<Tz>
    where
        Self: Sized,
        F: crate::job::AsyncHandler<Args> + Send + 'static + Copy,
        Args: Send + 'static + Clone,
        Tz: TimeZone + Send + Copy + 'static,
        <Tz as TimeZone>::Offset: Send,
    {
        for job in jobs {
            let job = job.with_tz(self.get_tz());
            self.add_job(Box::new(job));
        }
        self
    }

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
