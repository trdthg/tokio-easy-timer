use std::{fmt::Debug, marker::PhantomData, str::FromStr, time::Duration};
mod async_handler;
mod handler;

use cron::Schedule;

use crate::{
    extensions::{Data, DebugAny, Extensions},
    interval::Interval,
    scheduler::BoxedJob,
    Scheduler,
};

use self::async_handler::AsyncHandler;
use self::handler::Handler;
use async_trait::async_trait;

#[async_trait]
pub trait Job {
    async fn start_schedule(&self, e: Extensions);
}

pub struct SyncJob<Args, F> {
    f: F,
    jobschedules: Vec<JobSchedule>,
    _phantom: PhantomData<Args>,
}

#[derive(Clone)]
pub struct JobSchedule {
    since: chrono::DateTime<chrono::Local>,
    schedule: Schedule,
    repeat: u32,
    interval: u64,
}

pub struct JobScheduleBuilder {
    since: (u32, u32, u32),
    cron: Vec<Option<String>>,
    repeat: u32,
    interval: u64,
}

pub struct JobBuilder<Args, F> {
    f: Option<F>,
    jobschedules: Vec<JobSchedule>,
    builder: JobScheduleBuilder,
    _phantom: PhantomData<Args>,
}

macro_rules! every_start {
    ($( {$Varient: ident, $Index: expr} ),* | $({$VarientWeek: ident, $I: expr} ),* ) => {
        impl<Args, F> JobBuilder<Args, F> {

            /// since 只重复一次
            pub fn at(mut self, interval: Interval) -> Self {
                match interval {
                    $(
                        Interval::$Varient(x) => {
                            if let Some(s) = &self.builder.cron[$Index] {
                                self.builder.cron[$Index] = Some(format!("{},{}", s, x));
                            } else {
                                self.builder.cron[$Index] = Some(format!("{}", x));
                            }
                        }
                    )*
                    $(
                        Interval::$VarientWeek => {
                            if let Some(s) = &self.builder.cron[5] {
                                self.builder.cron[5] = Some(format!("{},{}", s, $I));
                            } else {
                                self.builder.cron[5] = Some(format!("{}", $I));
                            }
                        }
                    )*
                    Interval::Weekday => {
                        self.builder.cron[5] = Some("2-6".to_string());
                    }
                }
                self
            }

            /// 重复，带有起始时间
            pub fn since_every(mut self, start: Interval, interval: Interval) -> Self {
                match (start, interval) {
                    $(
                        (Interval::$Varient(start), Interval::$Varient(interval)) => {
                            if let Some(s) = &self.builder.cron[$Index] {
                                self.builder.cron[$Index] = Some(format!("{},{}/{}", s, start, interval));
                            } else {
                                self.builder.cron[$Index] = Some(format!("{}/{}", start, interval));
                            }
                        }
                    )*
                    _ => unimplemented!(),
                }
                self
            }

            // every 不带起始时间
            pub fn every(mut self, interval: Interval) -> Self {
                match interval {
                    $(
                        Interval::$Varient(x) => {
                            if let Some(s) = &self.builder.cron[$Index] {
                                self.builder.cron[$Index] = Some(format!("{},{}", s, format!("0/{}", x)))
                                // }
                            } else {
                                // 新增的，since 默认为 0
                                self.builder.cron[$Index] = Some(format!("0/{}", x));
                            }
                        }
                    )*
                    $(
                        Interval::$VarientWeek => {
                            // Sun
                            let week = format!("{}", $I);
                            if let Some(s) = &self.builder.cron[5] {
                                self.builder.cron[5] = Some(format!("{},{}", s, week));
                            } else {
                                self.builder.cron[5] = Some(week);
                            }
                        }
                    )*
                    Interval::Weekday => {
                        self.builder.cron[5] = Some("2-6".to_string());
                    }
                }
                self
            }

        }
    };
}

every_start!({Seconds, 0}, {Minutes, 1}, {Hours, 2}, {Days, 3}, {Weeks, 4} | { Sunday, 1 }, { Monday, 2},  { Tuesday, 3 }, { Wednesday, 4 }, { Thursday, 5 }, { Friday, 6 }, { Saturday, 7 });

impl<Args, F> JobBuilder<Args, F> {
    pub fn next(mut self) -> Self {
        self.jobschedules.push(self.builder.build());
        self.builder = JobScheduleBuilder::new();
        self
    }

    pub fn at_time(mut self, hour: u32, min: u32, sec: u32) -> Self {
        self.builder.cron[0] = Some(sec.to_string());
        self.builder.cron[1] = Some(min.to_string());
        self.builder.cron[2] = Some(hour.to_string());
        self
    }

    pub fn since_time(mut self, hour: u32, min: u32, sec: u32) -> Self {
        self.builder.since = (hour, min, sec);
        self
    }

    pub fn repeat(mut self, n: u32, interval: Interval) -> Self {
        self.builder.repeat = n;
        self.builder.interval = interval.to_sec();
        self
    }

    pub fn run(mut self, f: F) -> BoxedJob
    where
        Args: Clone + 'static + Debug + Send + Sync,
        F: Handler<Args> + Send + Sync + Copy + 'static,
    {
        self = self.next();
        self.f = Some(f);
        // self
        Box::new(SyncJob {
            f: self.f.unwrap(),
            _phantom: PhantomData,
            jobschedules: self.jobschedules,
        })
    }
}

#[async_trait]
impl<Args, F> Job for SyncJob<Args, F>
where
    F: Handler<Args> + Send + Sync + 'static + Clone + Copy,
    Args: Clone + 'static + Debug + Send + Sync,
{
    async fn start_schedule(&self, e: Extensions) {
        let mut han = vec![];
        for schedule in self.jobschedules.iter() {
            let f = self.f.clone();
            let e = e.clone();
            let schedule = schedule.clone();
            let t = tokio::spawn(async move {
                let now = chrono::Local::now();
                let wait_to = schedule.since;
                let d = wait_to - now;
                tokio::time::sleep(Duration::from_secs(d.num_seconds() as u64)).await;
                for next in schedule.schedule.upcoming(chrono::Local) {
                    let now = chrono::Local::now();
                    println!("{}", next);
                    println!("{}", now);
                    if next < now {
                        continue;
                    }
                    let d: i64 = next.timestamp() - now.timestamp();
                    println!("距离：{} 还差 {} 秒", next, d);

                    let f = f.clone();
                    let e = e.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(d as u64 - schedule.interval)).await;
                        for _ in 0..schedule.repeat {
                            tokio::time::sleep(Duration::from_secs(schedule.interval)).await;
                            let e = e.clone();
                            f.call(e);
                        }
                    });
                    tokio::time::sleep(Duration::from_secs(d as u64)).await;
                }
            });
            han.push(t);
        }
        for t in han {
            t.await.expect("msg");
        }
    }
}

impl<Args, F> SyncJob<Args, F> {
    pub fn new() -> JobBuilder<Args, F> {
        JobBuilder {
            f: None,
            _phantom: PhantomData,
            jobschedules: vec![],
            builder: JobScheduleBuilder::new(),
        }
    }
}
impl JobScheduleBuilder {
    fn new() -> Self {
        Self {
            since: (0, 0, 0),
            cron: vec![
                None, None, None, None, None, None,
                None,
                // "0".to_string(), // sec
                // "0".to_string(), // min
                // "0".to_string(), // hour
                // "*".to_string(), // day
                // "*".to_string(), // week
                // "*".to_string(), // week
                // "*".to_string(), // year
            ],
            repeat: 1,
            interval: 1,
        }
    }

    fn build(&self) -> JobSchedule {
        let mut cron = self.cron.clone();
        for i in 0..6 {
            if cron[i].is_some() && cron[i + 1].is_none() {
                cron[i + 1] = Some("*".to_string())
            }
        }
        let s = cron
            .iter()
            .enumerate()
            .map(|(i, x)| {
                // x.as_deref().unwrap_or("0")
                x.as_deref().unwrap_or_else(|| match i {
                    0 | 1 | 2 => "0",
                    _ => "*",
                })
            })
            .collect::<Vec<&str>>()
            .join(" ");

        println!("Cron: {}", s);
        let s = Schedule::from_str(s.as_str()).expect("cron 表达式不合法");
        JobSchedule {
            schedule: s,
            repeat: self.repeat as u32,
            interval: self.interval,
            since: chrono::Local::now(),
        }
    }
}
