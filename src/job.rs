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
    schedules: Vec<JobSchedule>,
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
    cron: Vec<String>,
    repeat: u32,
    interval: u64,
}

pub struct JobBuilder<Args, F> {
    f: Option<F>,
    schedules: Vec<JobSchedule>,
    builder: JobScheduleBuilder,
    _phantom: PhantomData<Args>,
}

macro_rules! every_start {
    ($( {$Varient: ident, $Index: expr} ),*) => {
        impl<Args, F> JobBuilder<Args, F> {
            pub fn every(mut self, interval: Interval) -> Self {
                match interval {
                    $(
                        Interval::$Varient(x) => {
                            if let Some((start, _)) = self.builder.cron[$Index].split_once("/") {
                                self.builder.cron[$Index] = format!("{}/{}", start, x);
                            } else {
                                self.builder.cron[$Index] = format!("0/{}", x);
                            }
                        }
                    )*
                    Interval::Sunday => {
                        // Sun
                        self.builder.cron[5] = "1".to_string();
                    }
                    Interval::Monday => {
                        // Mon
                        self.builder.cron[5] = "2".to_string();
                    }
                    Interval::Tuesday => {
                        // Tue
                        self.builder.cron[5] = "3".to_string();
                    }
                    Interval::Wednesday => {
                        // Wed
                        self.builder.cron[5] = "4".to_string();
                    }
                    Interval::Thursday => {
                        // Thu
                        self.builder.cron[5] = "5".to_string();
                    }
                    Interval::Friday => {
                        // Fri
                        self.builder.cron[5] = "6".to_string();
                    }
                    Interval::Saturday => {
                        // Sat
                        self.builder.cron[5] = "7".to_string();
                    }
                    Interval::Weekday => {
                        self.builder.cron[5] = "2-6".to_string();
                    }
                }
                self
            }

            pub fn since(mut self, interval: Interval) -> Self {
                match interval {
                    $(
                        Interval::$Varient(x) => {
                            if let Some((_, increase)) = self.builder.cron[$Index].split_once("/") {
                                self.builder.cron[$Index] = format!("{}/{}", x, increase);
                            } else {
                                self.builder.cron[$Index] = format!("{}", x);
                            }
                        }
                    )*
                    _ => unimplemented!(),
                }
                self
            }

        }
    };
}

every_start!({Seconds, 0}, {Minutes, 1}, {Hours, 2}, {Days, 3}, {Weeks, 4});

impl<Args, F> JobBuilder<Args, F> {
    pub fn next(mut self) -> Self {
        self.schedules.push(self.builder.build());
        self.builder = JobScheduleBuilder::new();
        self
    }

    pub fn at(mut self, interval: Interval) -> Self {
        match interval {
            Interval::Seconds(x) => {
                self.builder.cron[0] = x.to_string();
            }
            Interval::Minutes(x) => {
                self.builder.cron[1] = x.to_string();
            }
            Interval::Hours(x) => {
                self.builder.cron[2] = x.to_string();
            }
            _ => {}
        }
        self
    }

    pub fn at_time(mut self, hour: u32, min: u32, sec: u32) -> Self {
        self.builder.cron[0] = sec.to_string();
        self.builder.cron[1] = min.to_string();
        self.builder.cron[2] = hour.to_string();
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
            schedules: self.schedules,
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
        for shedule in self.schedules.iter() {
            let f = self.f.clone();
            let e = e.clone();
            let shedule = shedule.clone();
            let t = tokio::spawn(async move {
                let now = chrono::Local::now();
                let wait_to = shedule.since;
                let d = wait_to - now;
                tokio::time::sleep(Duration::from_secs(d.num_seconds() as u64)).await;
                for next in shedule.schedule.upcoming(chrono::Local) {
                    let now = chrono::Local::now();
                    // println!("{}", next);
                    // println!("{}", now);
                    if next < now {
                        continue;
                    }
                    let d: i64 = next.timestamp() - now.timestamp();
                    println!("还差 {} 秒", d);

                    let f = f.clone();
                    let e = e.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(d as u64 - shedule.interval)).await;
                        for _ in 0..shedule.repeat {
                            tokio::time::sleep(Duration::from_secs(shedule.interval)).await;
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
            schedules: vec![],
            builder: JobScheduleBuilder::new(),
        }
    }
}
impl JobScheduleBuilder {
    fn new() -> Self {
        Self {
            since: (0, 0, 0),
            cron: vec![
                "0".to_string(), // sec
                "0".to_string(), // min
                "0".to_string(), // hour
                "*".to_string(), // day
                "*".to_string(), // week
                "*".to_string(), // week
                "*".to_string(), // year
            ],
            repeat: 1,
            interval: 1,
        }
    }

    fn build(&self) -> JobSchedule {
        let s = self.cron.join(" ");
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
