use std::{marker::PhantomData, time::Duration};

use chrono::TimeZone;

use crate::{extensions::Extensions, interval::Interval, scheduler::BoxedJob};

use super::{
    jobschedule::{JobSchedule, JobScheduleBuilder},
    AsyncHandler, Job, JobBuilder,
};

#[derive(Clone)]
pub struct AsyncJob<Args, F> {
    pub f: F,
    pub jobschedules: Vec<JobSchedule>,
    pub _phantom: PhantomData<Args>,
}

pub struct AsyncJobBuilder<Args> {
    jobschedules: Vec<JobSchedule>,
    builder: JobScheduleBuilder,
    _phantom: PhantomData<Args>,
}

impl<Args, F, Tz> Job<Tz> for AsyncJob<Args, F>
where
    F: AsyncHandler<Args> + Send + 'static + Copy,
    Args: Send + 'static + Clone,
    Tz: TimeZone + Send + Copy + 'static,
    <Tz as TimeZone>::Offset: Send,
{
    fn box_clone(&self) -> Box<(dyn Job<Tz> + Send + 'static)> {
        Box::new((*self).clone())
    }

    fn start_schedule(&self, e: Extensions, tz: Tz) {
        for schedule in self.jobschedules.iter() {
            // spawn a task for every corn schedule
            let f = self.f.clone();
            let e = e.clone();
            let schedule = schedule.clone();
            tokio::spawn(async move {
                // delay
                let now = chrono::Local::now().with_timezone(&tz);
                let since = schedule.since;
                let wait_to = tz
                    .ymd(since.0, since.1, since.2)
                    .and_hms(since.3, since.4, since.5);
                let d = wait_to.timestamp() - now.timestamp();
                if d > 0 {
                    tokio::time::sleep(Duration::from_secs(d as u64)).await;
                }

                // run jobs
                for next in schedule.schedule.upcoming(tz) {
                    // Calculates the time left until the next task run
                    let now = chrono::Local::now().with_timezone(&tz);
                    let d = next.timestamp() - now.timestamp();
                    if d < 0 {
                        continue;
                    }
                    let e = e.clone();
                    // prepare a task for the next job
                    tokio::spawn(async move {
                        // Wait until the next job runs
                        tokio::time::sleep(Duration::from_secs(d as u64)).await;

                        // Handle repeat
                        for i in 0..schedule.repeat {
                            if schedule.is_async {
                                let e = e.clone();
                                tokio::spawn(async move {
                                    f.call(&e).await;
                                });
                            } else {
                                f.call(&e).await;
                            }
                            if schedule.interval > 0 && i < schedule.repeat - 1 {
                                tokio::time::sleep(Duration::from_secs(schedule.interval)).await;
                            }
                        }
                    });

                    // wait until this task run
                    tokio::time::sleep(Duration::from_secs(d as u64)).await;
                }
            });
        }
    }
}

impl<Args> AsyncJobBuilder<Args>
where
    Args: Clone + 'static + Send + Sync,
{
    /// Constructs a new async job
    pub fn run<Tz, F>(&mut self, f: F) -> BoxedJob<Tz>
    where
        F: AsyncHandler<Args> + Send + 'static + Clone + Copy,
        Tz: TimeZone + Send + Sync + 'static + Clone + Copy,
        <Tz as TimeZone>::Offset: Send + Sync,
    {
        self.and();
        let job: AsyncJob<Args, F> = AsyncJob {
            f: f.to_owned(),
            jobschedules: self.jobschedules.clone(),
            _phantom: PhantomData,
        };
        let res = Box::new(job);
        res
    }

    // / Constructs a new async job
    // pub fn build<Tz, F>(&mut self, f: F) -> AsyncJob<Args, F>
    // where
    //     F: AsyncHandler<Args> + Send + 'static + Clone + Copy,
    //     Tz: TimeZone + Send + Sync + 'static + Clone + Copy,
    //     <Tz as TimeZone>::Offset: Send + Sync,
    // {
    //     self.and();
    //     let job: AsyncJob<Args, F> = AsyncJob {
    //         f: f.to_owned(),
    //         jobschedules: self.jobschedules.clone(),
    //         _phantom: PhantomData,
    //     };
    //     job
    // }
}

impl<Args> JobBuilder<Args> for AsyncJobBuilder<Args> {
    fn new() -> Self {
        Self {
            _phantom: PhantomData,
            jobschedules: vec![],
            builder: JobScheduleBuilder::new(),
        }
    }

    fn and(&mut self) -> &mut Self {
        self.jobschedules.push(self.builder.build());
        self.builder = JobScheduleBuilder::new();
        self
    }

    fn get_mut_cron_builder(&mut self) -> &mut JobScheduleBuilder {
        &mut self.builder
    }

    fn get_mut_since(&mut self) -> &mut (i32, u32, u32, u32, u32, u32) {
        &mut self.builder.since
    }

    fn repeat_seq(&mut self, n: u32, interval: Interval) -> &mut Self {
        self.builder.is_async = false;
        self.builder.repeat = n;
        self.builder.interval = interval.to_sec();
        self
    }

    fn repeat_async(&mut self, n: u32, interval: Interval) -> &mut Self {
        self.builder.is_async = true;
        self.builder.repeat = n;
        self.builder.interval = interval.to_sec();
        self
    }
}
