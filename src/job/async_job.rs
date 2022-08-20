use std::{future::Future, marker::PhantomData, pin::Pin, time::Duration};

use chrono::{Datelike, TimeZone};

use crate::{
    extensions::Extensions,
    interval::Interval,
    scheduler::{item::ScheduleItem, BoxedJob},
};

use super::{
    jobschedule::{JobSchedule, JobScheduleBuilder},
    AsyncHandler, Job, JobBuilder, JobId,
};

#[derive(Clone)]
pub struct AsyncJob<Args, F> {
    pub id: JobId,
    pub f: F,
    pub cancel: bool,
    pub jobschedules: Vec<JobSchedule>,
    pub current_time: u64,
    pub next_schedule_index: usize,
    pub _phantom: PhantomData<Args>,
}

pub struct AsyncJobBuilder<Args> {
    id: JobId,
    jobschedules: Vec<JobSchedule>,
    builder: JobScheduleBuilder,
    _phantom: PhantomData<Args>,
}

#[async_trait::async_trait]
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

    // fn start_schedule(&self, e: Extensions, tz: Tz) {
    //     for schedule in self.jobschedules.iter() {
    //         // spawn a task for every corn schedule
    //         let f = self.f.clone();
    //         let e = e.clone();
    //         let schedule = schedule.clone();
    //         tokio::spawn(async move {
    //             // delay
    //             let now = chrono::Local::now().with_timezone(&tz);
    //             let since = schedule.since;
    //             let wait_to = tz
    //                 .ymd(since.0, since.1, since.2)
    //                 .and_hms(since.3, since.4, since.5);
    //             let d = wait_to.timestamp() - now.timestamp();
    //             if d > 0 {
    //                 tokio::time::sleep(Duration::from_secs(d as u64)).await;
    //             }

    //             // run jobs
    //             for next in schedule.schedule.upcoming(tz) {
    //                 // Calculates the time left until the next task run
    //                 let now = chrono::Local::now().with_timezone(&tz);
    //                 let d = next.timestamp() - now.timestamp();
    //                 if d < 0 {
    //                     continue;
    //                 }
    //                 let e = e.clone();
    //                 // prepare a task for the next job
    //                 tokio::spawn(async move {
    //                     // Wait until the next job runs
    //                     tokio::time::sleep(Duration::from_secs(d as u64)).await;

    //                     // Handle repeat
    //                     for i in 0..schedule.repeat {
    //                         if schedule.is_async {
    //                             let e = e.clone();
    //                             tokio::spawn(async move {
    //                                 f.call(&e).await;
    //                             });
    //                         } else {
    //                             f.call(&e).await;
    //                         }
    //                         if schedule.interval > 0 && i < schedule.repeat - 1 {
    //                             tokio::time::sleep(Duration::from_secs(schedule.interval)).await;
    //                         }
    //                     }
    //                 });

    //                 // wait until this task run
    //                 tokio::time::sleep(Duration::from_secs(d as u64)).await;
    //             }
    //         });
    //     }
    // }

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
        Some(ScheduleItem {
            id: self.id,
            time: min,
        })
    }

    fn next_job(&mut self, tz: Tz) -> Option<crate::scheduler::item::ScheduleJobItem<Tz>> {
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
        Some(crate::scheduler::item::ScheduleJobItem {
            time: min,
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
            id: self.id,
            f: f.to_owned(),
            jobschedules: self.jobschedules.clone(),
            _phantom: PhantomData,
            cancel: false,
            current_time: 0,
            next_schedule_index: 0,
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
            id: JobId(0),
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

    fn get_mut_since(&mut self) -> &mut (Option<(i32, u32, u32)>, Option<(u32, u32, u32)>) {
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
