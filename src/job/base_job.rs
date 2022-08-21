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
    AsyncHandler, AsyncJob, Job, JobBuilder, JobId, SyncHandler, SyncJob,
};

#[derive(Clone)]
pub struct BaseJob {
    pub id: JobId,
    pub cancel: bool,
    pub jobschedule: JobSchedule,
    pub current_time: u64,
    pub next_schedule_index: usize,
}

pub struct BaseJobBuilder {
    id: JobId,
    jobschedules: Option<Vec<JobSchedule>>,
    builder: JobScheduleBuilder,
}

impl BaseJob {
    fn current(&mut self) -> Option<ScheduleItem> {
        Some(ScheduleItem {
            id: self.id,
            time: self.current_time,
        })
    }

    fn next<Tz: TimeZone>(&mut self, tz: Tz) -> Option<ScheduleItem> {
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

impl BaseJobBuilder {
    pub fn run_sync<Tz, F, Args>(&mut self, f: F) -> Vec<BoxedJob<Tz>>
    where
        F: SyncHandler<Args> + Send + 'static + Copy,
        Args: Clone + 'static + Send + Sync,
        Tz: TimeZone + Clone + Send + Copy + 'static,
        <Tz as TimeZone>::Offset: Send,
    {
        self.and();
        let schedules = self.jobschedules.take().unwrap_or_default().into_iter();
        let mut res = vec![];
        for x in schedules.into_iter() {
            let job: BoxedJob<Tz> = Box::new(SyncJob {
                id: self.id,
                f: f.clone(),
                cancel: false,
                jobschedule: x,
                _phantom: PhantomData,
                current_time: 0,
                next_schedule_index: 0,
            });
            res.push(job);
        }
        res
    }

    pub fn run_async<Tz, F, Args>(&mut self, f: F) -> Vec<BoxedJob<Tz>>
    where
        F: AsyncHandler<Args> + Send + 'static + Copy,
        Args: Clone + 'static + Send + Sync,
        Tz: TimeZone + Clone + Send + Copy + 'static,
        <Tz as TimeZone>::Offset: Send,
    {
        self.and();
        let schedules = self.jobschedules.take().unwrap_or_default().into_iter();
        let mut res = vec![];
        for x in schedules.into_iter() {
            let job: BoxedJob<Tz> = Box::new(AsyncJob {
                id: self.id,
                f: f.clone(),
                cancel: false,
                jobschedule: x,
                _phantom: PhantomData,
                current_time: 0,
                next_schedule_index: 0,
            });
            res.push(job);
        }
        res
    }
}

impl JobBuilder for BaseJobBuilder {
    fn new() -> Self {
        Self {
            builder: JobScheduleBuilder::new(),
            id: JobId(0),
            jobschedules: None,
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

    fn get_mut_cron_builder(&mut self) -> &mut JobScheduleBuilder {
        &mut self.builder
    }

    fn get_mut_since(&mut self) -> &mut (Option<(i32, u32, u32)>, Option<(u32, u32, u32)>) {
        &mut self.builder.since
    }
}
