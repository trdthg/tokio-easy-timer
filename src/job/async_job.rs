use std::{future::Future, marker::PhantomData, pin::Pin, time::Duration};

use chrono::{DateTime, Datelike, TimeZone};

use crate::{
    extensions::Extensions,
    interval::Interval,
    scheduler::{item::ScheduleItem, BoxedJob},
};

use super::{
    base_job::BaseJob,
    jobschedule::{JobSchedule, JobScheduleBuilder},
    AsyncHandler, Job, JobBuilder, JobId,
};

pub struct AsyncJob<Args, F, Tz: TimeZone> {
    pub tz: Tz,
    pub f: F,
    pub _phantom: PhantomData<Args>,
    pub info: BaseJob,
    pub iter: cron::OwnedScheduleIterator<Tz>,
}

#[derive(Clone)]
pub struct AsyncJobBuilder<Args, F> {
    pub f: F,
    pub _phantom: PhantomData<Args>,
    pub info: BaseJob,
}

impl<Args, F> AsyncJobBuilder<Args, F>
where
    F: AsyncHandler<Args> + Send + 'static + Copy,
    Args: Send + 'static + Clone,
{
    pub fn with_tz<Tz: TimeZone>(&self, tz: Tz) -> AsyncJob<Args, F, Tz> {
        AsyncJob {
            tz: tz.clone(),
            f: self.f.clone(),
            _phantom: PhantomData::default(),
            info: self.info.clone(),
            iter: self.info.jobschedule.schedule.upcoming_owned(tz),
        }
    }
}

#[async_trait::async_trait]
impl<Args, F, Tz> Job<Tz> for AsyncJob<Args, F, Tz>
where
    F: AsyncHandler<Args> + Send + 'static + Copy,
    Args: Send + 'static + Clone,
    Tz: TimeZone + Send + Copy + 'static,
    <Tz as TimeZone>::Offset: Send,
{
    fn box_clone(&self) -> Box<(dyn Job<Tz> + Send + 'static)> {
        Box::new(AsyncJob {
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

    fn next(&mut self, tz: Tz) -> Option<ScheduleItem> {
        if let Some(next_time) = self.iter.next().and_then(|x| Some(x.timestamp() as u64)) {
            Some(ScheduleItem {
                id: self.info.id,
                time: next_time,
            })
        } else {
            None
        }
    }

    fn next_job(&mut self, tz: Tz) -> Option<crate::scheduler::item::ScheduleJobItem<Tz>> {
        self.info.next(tz);
        Some(crate::scheduler::item::ScheduleJobItem {
            time: self.info.current_time,
            job: self.box_clone(),
        })
    }

    fn run<'a: 'b, 'b>(
        &'a self,
        e: Extensions,
        tz: Tz,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>> {
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
        })
    }

    fn get_id(&self) -> JobId {
        self.info.id
    }

    fn set_id(&mut self, id: JobId) {
        self.info.id = id;
    }
}
