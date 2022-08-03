use std::marker::PhantomData;

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use chrono::TimeZone;

use crate::{extensions::Extensions, interval::Interval, scheduler::BoxedJob};

use super::{
    jobschedule::{JobSchedule, JobScheduleBuilder},
    Job, JobBuilder, SyncHandler,
};

#[derive(Clone)]
pub struct SyncJob<Args, F> {
    pub f: F,
    pub jobschedules: Vec<JobSchedule>,
    pub _phantom: PhantomData<Args>,
}

pub struct SyncJobBuilder<Args> {
    jobschedules: Vec<JobSchedule>,
    builder: JobScheduleBuilder,
    _phantom: PhantomData<Args>,
}

#[async_trait]
impl<Args, F, Tz> Job<Tz> for SyncJob<Args, F>
where
    Tz: TimeZone + Send + Sync + Clone + Copy + 'static,
    <Tz as TimeZone>::Offset: Send + Sync,
    F: SyncHandler<Args> + Send + Sync + 'static + Copy,
    Args: Clone + 'static + Send + Sync,
{
    async fn start_schedule(&self, e: Extensions, tz: Tz) {
        for schedule in self.jobschedules.iter() {
            // spawn a task for every corn schedule
            let f = self.f.clone();
            let e = e.clone();
            let schedule = schedule.clone();
            tokio::spawn(async move {
                // delay
                if schedule.delay > 0 {
                    tokio::time::sleep(Duration::from_secs(schedule.delay)).await;
                }
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
                    let e = e.clone();

                    // Calculates the time left until the next job run
                    let now = chrono::Local::now().with_timezone(&tz);
                    let d = next.timestamp() - now.timestamp();
                    if d < 0 {
                        continue;
                    }

                    // prepare a task for the job
                    tokio::spawn(async move {
                        // Wait until the next job runs
                        tokio::time::sleep(Duration::from_secs(d as u64)).await;

                        // Handle repeat
                        if schedule.is_async {
                            for i in 0..schedule.repeat {
                                let e = e.clone();
                                tokio::task::spawn_blocking(move || {
                                    f.call(&e);
                                });
                                if i < schedule.repeat - 1 {
                                    tokio::time::sleep(Duration::from_secs(schedule.interval))
                                        .await;
                                }
                            }
                        } else {
                            tokio::task::spawn_blocking(move || {
                                for i in 0..schedule.repeat {
                                    f.call(&e);
                                    if schedule.interval > 0 && i < schedule.repeat - 1 {
                                        std::thread::sleep(Duration::from_secs(schedule.interval));
                                    }
                                }
                            });
                        }
                    });

                    // wait until this task run
                    tokio::time::sleep(Duration::from_secs(d as u64)).await;
                }
            });
        }
    }
}

impl<Args> SyncJobBuilder<Args>
where
    Args: Clone + 'static + Send + Sync,
{
    /// Constructs a new sync job
    pub fn run<Tz, F>(&mut self, f: F) -> BoxedJob<Tz>
    where
        F: SyncHandler<Args> + Copy + Send + Sync + 'static,
        Tz: TimeZone + Clone + Send + Sync + Copy + 'static,
        <Tz as TimeZone>::Offset: Send + Sync,
    {
        self.and();
        Arc::new(SyncJob {
            f,
            jobschedules: self.jobschedules.clone(),
            _phantom: PhantomData,
        })
    }
}

impl<Args> JobBuilder<Args> for SyncJobBuilder<Args> {
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
