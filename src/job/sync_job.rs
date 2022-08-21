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
    pub jobschedule: JobSchedule,
    pub current_time: u64,
    pub next_schedule_index: usize,
    pub _phantom: PhantomData<Args>,
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

        let mut min = u64::MAX;

        if let Some(next_time) = self
            .jobschedule
            .schedule
            .upcoming(tz)
            .take(1)
            .next()
            .and_then(|x| Some(x.timestamp() as u64))
        {
            self.current_time = next_time;
            Some(ScheduleItem {
                id: self.id,
                time: self.current_time,
            })
        } else {
            None
        }
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
        let schedule = self.jobschedule.clone();
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
