use std::{
    future::Future,
    hash::Hash,
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, Mutex},
};

use std::time::Duration;

use chrono::{Datelike, TimeZone};

use crate::{
    extensions::Extensions,
    interval::Interval,
    scheduler::{item::ScheduleItem, BoxedJob},
};

use super::{
    jobschedule::{self, JobSchedule, JobScheduleBuilder},
    Job, JobBuilder, JobId, SyncHandler,
};

#[derive(Clone)]
pub struct SyncJob<Args, F> {
    pub id: JobId,
    pub f: F,
    pub cancel: bool,
    pub jobschedules: Vec<Arc<JobSchedule>>,
    pub current_time: u64,
    pub next_schedule_index: usize,
    pub _phantom: PhantomData<Args>,
}

pub struct SyncJobBuilder<Args> {
    id: JobId,
    jobschedules: Option<Vec<JobSchedule>>,
    builder: JobScheduleBuilder,
    _phantom: PhantomData<Args>,
}
use async_trait::async_trait;
#[async_trait]
impl<Args, F, Tz> Job<Tz> for SyncJob<Args, F>
where
    F: SyncHandler<Args> + Send + 'static + Copy,
    Args: Send + 'static + Clone,
    Tz: TimeZone + Send + Copy + 'static,
    <Tz as TimeZone>::Offset: Send,
{
    fn box_clone(&self) -> Box<(dyn Job<Tz> + Send + 'static)> {
        Box::new((*self).clone())
    }

    fn current(&mut self) -> Option<ScheduleItem> {
        Some(ScheduleItem {
            id: self.id,
            time: self.current_time,
        })
    }

    fn next(&mut self, tz: Tz) -> Option<ScheduleItem> {
        if self.cancel {
            return None;
        }
        if self.jobschedules.len() == 0 {
            return None;
        }

        let mut min = u64::MAX;

        let next_times: Vec<(usize, u64)> = self
            .jobschedules
            .iter()
            .enumerate()
            .filter_map(|(i, x)| {
                x.schedule
                    .upcoming(tz)
                    .take(1)
                    .next()
                    .and_then(|x| Some((i, x.timestamp() as u64)))
            })
            .collect();

        let mut min_index = 0;
        for (i, time) in next_times.iter() {
            if *time < min {
                min = *time;
                min_index = *i;
            }
        }
        self.current_time = min;
        self.next_schedule_index = min_index;
        let min = chrono::Local::now().timestamp() as u64 + 10;
        Some(ScheduleItem {
            id: self.id,
            time: self.current_time,
        })
    }

    fn next_job(&mut self, tz: Tz) -> Option<crate::scheduler::item::ScheduleJobItem<Tz>> {
        self.next(tz);
        Some(crate::scheduler::item::ScheduleJobItem {
            time: self.current_time,
            job: Box::new(self.clone()),
        })
    }

    fn run<'a: 'b, 'b>(
        &'a self,
        e: Extensions,
        tz: Tz,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>> {
        let f = self.f.clone();
        let schedule = self.jobschedules[self.next_schedule_index].clone();
        Box::pin(async move {
            // handle delay
            // println!("delay {}", schedule.delay);
            if schedule.delay > 0 {
                tokio::time::sleep(Duration::from_secs(schedule.delay)).await;
            }
            // handle since
            // if let Some(delay) = schedule.get_since_delay_sec(tz) {
            //     tokio::time::sleep(Duration::from_secs(delay as u64)).await;
            // }

            let e = e.clone();
            // prepare a task for the job
            tokio::spawn(async move {
                // Handle repeat
                if schedule.is_async {
                    for i in 0..schedule.repeat {
                        let e = e.clone();
                        tokio::task::spawn_blocking(move || {
                            f.call(&e);
                        });
                        if i < schedule.repeat - 1 {
                            tokio::time::sleep(Duration::from_secs(schedule.interval)).await;
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
        })
    }

    fn get_id(&self) -> JobId {
        self.id
    }

    fn set_id(&mut self, id: JobId) {
        self.id = id;
    }

    fn start(&mut self) {
        self.cancel = false;
    }

    fn stop(&mut self) {
        self.cancel = true;
    }
}

impl<Args> SyncJobBuilder<Args>
where
    Args: Clone + 'static + Send,
{
    /// Constructs a new sync job
    pub fn run<Tz, F>(&mut self, f: F) -> BoxedJob<Tz>
    where
        F: SyncHandler<Args> + Send + 'static + Copy,
        Tz: TimeZone + Clone + Send + Copy + 'static,
        <Tz as TimeZone>::Offset: Send,
    {
        self.and();

        let jobschedules = self
            .jobschedules
            .take()
            .unwrap_or_default()
            .into_iter()
            .map(|x| Arc::new(x))
            .collect();

        let res = Box::new(SyncJob {
            id: self.id,
            f,
            cancel: false,
            jobschedules,
            _phantom: PhantomData,
            current_time: 0,
            next_schedule_index: 0,
        });
        res
    }
}

impl<Args> JobBuilder<Args> for SyncJobBuilder<Args> {
    fn new() -> Self {
        Self {
            _phantom: PhantomData,
            jobschedules: None,
            builder: JobScheduleBuilder::new(),
            id: JobId(0),
        }
    }

    fn and(&mut self) -> &mut Self {
        if self.jobschedules.is_none() {
            self.jobschedules = Some(vec![]);
        }
        if let Some(jobschedules) = &mut self.jobschedules {
            jobschedules.push(self.builder.build());
        }
        self.builder = JobScheduleBuilder::new();
        self
    }

    fn get_mut_cron_builder(&mut self) -> &mut JobScheduleBuilder {
        &mut self.builder
    }

    fn get_mut_since(&mut self) -> &mut (Option<(i32, u32, u32)>, Option<(u32, u32, u32)>) {
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
