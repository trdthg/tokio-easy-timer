use std::marker::PhantomData;

use chrono::TimeZone;

use crate::{interval::Interval, scheduler::task::ScheduleItem};

use super::{
    async_job::AsyncJobBuilder,
    jobschedule::{JobSchedule, JobScheduleBuilder},
    sync_job::SyncJobBuilder,
    AsyncHandler, JobBuilder, JobId, SyncHandler,
};

#[derive(Clone)]
pub struct BaseJob {
    pub id: JobId,
    pub cancel: bool,
    pub jobschedule: JobSchedule,
    pub current_time: u64,
}

impl BaseJob {
    pub fn new(jobschedule: JobSchedule) -> Self {
        Self {
            id: JobId(0),
            cancel: false,
            jobschedule,
            current_time: 0,
        }
    }

    pub fn current(&mut self) -> Option<ScheduleItem> {
        Some(ScheduleItem {
            id: self.id,
            time: self.current_time,
        })
    }

    pub fn next<Tz: TimeZone>(&mut self, tz: Tz) -> Option<ScheduleItem> {
        if self.cancel {
            return None;
        }

        let _min = u64::MAX;

        if let Some(next_time) = self
            .jobschedule
            .schedule
            .upcoming(tz)
            .take(1)
            .next()
            .map(|x| x.timestamp() as u64)
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

    // fn get_id(&self) -> JobId {
    //     self.id
    // }

    // fn set_id(&mut self, id: JobId) {
    //     self.id = id;
    // }

    // fn start(&mut self) {
    //     self.cancel = false;
    // }

    // fn stop(&mut self) {
    //     self.cancel = true;
    // }
}

pub struct BaseJobBuilder {
    jobschedules: Option<Vec<JobSchedule>>,
    builder: JobScheduleBuilder,
}

impl BaseJobBuilder {
    pub fn run_sync<F, Args>(&mut self, f: F) -> Vec<SyncJobBuilder<Args, F>>
    where
        F: SyncHandler<Args> + Send + 'static + Copy,
        Args: Clone + 'static + Send + Sync,
    {
        self.and();
        let schedules = self.jobschedules.take().unwrap_or_default().into_iter();
        let mut res = vec![];
        for x in schedules.into_iter() {
            let job = SyncJobBuilder {
                f,
                _phantom: PhantomData,
                info: BaseJob::new(x),
            };
            res.push(job);
        }
        res
    }

    pub fn run_async<F, Args>(&mut self, f: F) -> Vec<AsyncJobBuilder<Args, F>>
    where
        F: AsyncHandler<Args> + Send + 'static + Copy,
        Args: Clone + 'static + Send + Sync,
    {
        self.and();
        let schedules = self.jobschedules.take().unwrap_or_default().into_iter();
        let mut res = vec![];
        for x in schedules.into_iter() {
            let job = AsyncJobBuilder {
                f,
                _phantom: PhantomData,
                info: BaseJob::new(x),
            };
            res.push(job);
        }
        res
    }
}

impl JobBuilder for BaseJobBuilder {
    fn new() -> Self {
        Self {
            builder: JobScheduleBuilder::new(),
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
