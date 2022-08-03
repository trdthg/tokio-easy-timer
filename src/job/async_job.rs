use std::{marker::PhantomData, sync::Arc, time::Duration};

use async_trait::async_trait;
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

#[async_trait]
impl<Args, F, Tz> Job<Tz> for AsyncJob<Args, F>
where
    Tz: TimeZone + Clone + Send + Sync + Copy + 'static,
    <Tz as TimeZone>::Offset: Send + Sync,
    F: AsyncHandler<Args> + Send + Sync + 'static + Copy,
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

                    let f = f.clone();
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
        F: AsyncHandler<Args> + Copy + Send + Sync + 'static,
        Tz: TimeZone + Clone + Send + Sync + Copy + 'static,
        <Tz as TimeZone>::Offset: Send + Sync,
    {
        self.and();
        Arc::new(AsyncJob {
            f,
            jobschedules: self.jobschedules.clone(),
            _phantom: PhantomData,
        })
    }
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

    fn get_cron_builder(&mut self) -> &mut JobScheduleBuilder {
        &mut self.builder
    }

    fn at_datetime(
        &mut self,
        year: Option<i32>,
        month: Option<u32>,
        day: Option<u32>,
        hour: Option<u32>,
        min: Option<u32>,
        sec: Option<u32>,
    ) -> &mut Self {
        self.builder.cron[0] = sec.and_then(|x| Some(x.to_string()));
        self.builder.cron[1] = min.and_then(|x| Some(x.to_string()));
        self.builder.cron[2] = hour.and_then(|x| Some(x.to_string()));
        self.builder.cron[3] = day.and_then(|x| Some(x.to_string()));
        self.builder.cron[4] = month.and_then(|x| Some(x.to_string()));
        self.builder.cron[6] = year.and_then(|x| Some(x.to_string()));
        self
    }

    fn since_datetime(
        &mut self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> &mut Self {
        self.builder.since = (year, month, day, hour, min, sec);
        self
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
