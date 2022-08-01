use std::marker::PhantomData;

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use chrono::TimeZone;

use crate::{extensions::Extensions, scheduler::BoxedJob};

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
    Tz: TimeZone + Clone + Send + Sync + Copy + 'static,
    <Tz as TimeZone>::Offset: Send + Sync,
    F: SyncHandler<Args> + Send + Sync + 'static + Clone + Copy,
    Args: Clone + 'static + Send + Sync,
{
    async fn start_schedule(&self, e: Extensions, tz: Tz) {
        let mut han = vec![];
        for schedule in self.jobschedules.iter() {
            // 为每一个任务运行的时间点单独 spawn 一个任务
            let f = self.f.clone();
            let e = e.clone();
            let schedule = schedule.clone();
            let t = tokio::spawn(async move {
                // 延时
                let now = chrono::Local::now().with_timezone(&tz);
                let since = schedule.since;
                let wait_to = tz
                    .ymd(since.0, since.1, since.2)
                    .and_hms(since.3, since.4, since.5);
                if wait_to > now {
                    let d = wait_to - now;
                    tokio::time::sleep(Duration::from_secs(d.num_seconds() as u64)).await;
                }

                // 开始执行任务
                for next in schedule.schedule.upcoming(tz) {
                    let e = e.clone();
                    // 计算距离下次任务运行剩余的时间
                    let now = chrono::Local::now().with_timezone(&tz);
                    let d = next.timestamp() - now.timestamp();
                    if d < 0 {
                        continue;
                    }
                    // println!("{:?}", next);
                    // println!("{:?}", now);
                    // println!("距离：{:?} 还差 {} 秒", next, d);

                    // 开启新线程执行用户任务
                    tokio::time::sleep(Duration::from_secs(d as u64)).await;
                    // TODO!

                    tokio::spawn(async move {
                        // 一直等待到下次任务运行
                        tokio::time::sleep(Duration::from_secs(d as u64)).await;

                        // 根据重复次数和间隔时间依次运行
                        if schedule.is_async {
                            for i in 0..schedule.repeat {
                                let e = e.clone();
                                tokio::task::spawn_blocking(move || {
                                    f.call(&e);
                                });
                                // 最后一次就不 sleep 了
                                if i < schedule.repeat - 1 {
                                    tokio::time::sleep(Duration::from_secs(schedule.interval))
                                        .await;
                                }
                            }
                        } else {
                            tokio::task::spawn_blocking(move || {
                                for i in 0..schedule.repeat {
                                    f.call(&e);
                                    // 最后一次就不 sleep 了
                                    if schedule.interval > 0 && i < schedule.repeat - 1 {
                                        std::thread::sleep(Duration::from_secs(schedule.interval));
                                    }
                                }
                            });
                        }
                    });
                }
            });
            han.push(t);
        }
        for t in han {
            t.await.expect("msg");
        }
    }
}

impl<Args> SyncJobBuilder<Args>
where
    Args: Clone + 'static + Send + Sync,
{
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

    fn repeat_seq(&mut self, n: u32, interval: crate::Interval) -> &mut Self {
        self.builder.repeat = n;
        self.builder.interval = interval.to_sec();
        self
    }

    fn repeat_async(&mut self, n: u32, interval: crate::Interval) -> &mut Self {
        self.builder.is_async = true;
        self.builder.repeat = n;
        self.builder.interval = interval.to_sec();
        self
    }
}
