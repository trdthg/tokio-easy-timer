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
    base_job::BaseJob,
    jobschedule::{self, JobSchedule, JobScheduleBuilder},
    Job, JobBuilder, JobId, SyncHandler,
};

pub struct SyncJob<Args, F, Tz: TimeZone> {
    pub tz: Tz,
    pub f: F,
    pub _phantom: PhantomData<Args>,
    pub info: BaseJob,
    pub iter: cron::OwnedScheduleIterator<Tz>,
}

pub struct SyncJobBuilder<Args, F> {
    pub f: F,
    pub _phantom: PhantomData<Args>,
    pub info: BaseJob,
}

impl<Args, F> SyncJobBuilder<Args, F>
where
    F: SyncHandler<Args> + Send + 'static + Copy,
    Args: Send + 'static + Clone,
{
    pub fn with_tz<Tz: TimeZone>(&self, tz: Tz) -> SyncJob<Args, F, Tz> {
        SyncJob {
            tz: tz.clone(),
            f: self.f.clone(),
            _phantom: PhantomData::default(),
            info: self.info.clone(),
            iter: self.info.jobschedule.schedule.upcoming_owned(tz),
        }
    }
}

use async_trait::async_trait;
#[async_trait]
impl<Args, F, Tz> Job<Tz> for SyncJob<Args, F, Tz>
where
    F: SyncHandler<Args> + Send + 'static + Copy,
    Args: Send + 'static + Clone,
    Tz: TimeZone + Send + Copy + 'static,
    <Tz as TimeZone>::Offset: Send,
{
    fn box_clone(&self) -> Box<(dyn Job<Tz> + Send + 'static)> {
        Box::new(SyncJob {
            tz: self.tz.clone(),
            f: self.f.clone(),
            _phantom: PhantomData::default(),
            info: self.info.clone(),
            iter: self
                .info
                .jobschedule
                .schedule
                .upcoming_owned(self.tz.clone()),
        })
    }

    fn next(&mut self) -> Option<ScheduleItem> {
        // self.info.next(tz)
        if let Some(next_time) = self.iter.next().and_then(|x| Some(x.timestamp() as u64)) {
            Some(ScheduleItem {
                id: self.info.id,
                time: next_time,
            })
        } else {
            None
        }
    }

    fn next_job(&mut self) -> Option<crate::scheduler::item::ScheduleJobItem<Tz>> {
        if let Some(next_time) = self.iter.next().and_then(|x| Some(x.timestamp() as u64)) {
            Some(crate::scheduler::item::ScheduleJobItem {
                time: next_time,
                job: self.box_clone(),
            })
        } else {
            None
        }
    }

    fn run<'a: 'b, 'b>(&'a self, e: Extensions) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>> {
        let f = self.f.clone();
        let schedule = self.info.jobschedule.clone();
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
        self.info.id
    }

    fn set_id(&mut self, id: JobId) {
        self.info.id = id;
    }
}
