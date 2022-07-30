use std::{fmt::Debug, marker::PhantomData, str::FromStr, time::Duration};
mod async_handler;
mod handler;

use cron::Schedule;

use crate::{
    extensions::{Data, DebugAny, Extensions},
    interval::Interval,
    Sheduler,
};

use self::async_handler::AsyncHandler;
use self::handler::Handler;
use async_trait::async_trait;

#[async_trait]
pub trait Job {
    async fn start_schedule(&self, e: Extensions);
}

pub struct SyncJob<Args, F> {
    pub f: F,
    pub schedules: Vec<Schedule>,
    repeat: usize,
    _phantom: PhantomData<Args>,
}

pub struct JobBuilder<Args, F> {
    f: Option<F>,
    schedules: Vec<Schedule>,
    tmp_interval: Vec<String>,
    repeat: Option<usize>,
    _phantom: PhantomData<Args>,
}

impl<Args, F> JobBuilder<Args, F> {
    pub fn build(self) -> crate::scheduler::BoxedJob
    where
        Args: Clone + 'static + Debug + Send + Sync,
        F: Handler<Args> + Send + Sync + Copy + 'static,
    {
        Box::new(SyncJob {
            f: self.f.unwrap(),
            repeat: self.repeat.unwrap(),
            _phantom: PhantomData,
            schedules: self.schedules,
        })
    }

    pub fn every(mut self, interval: Interval) -> Self {
        match interval {
            Interval::Days(x) => {
                self.tmp_interval[3] = x.to_string();
            }
            Interval::Weeks(x) => {
                self.tmp_interval[4] = x.to_string();
            }
            Interval::Monday => {
                self.tmp_interval[5] = "Mon".to_string();
            }
            Interval::Tuesday => {
                self.tmp_interval[5] = "Tue".to_string();
            }
            Interval::Wednesday => {
                self.tmp_interval[5] = "Wed".to_string();
            }
            Interval::Thursday => {
                self.tmp_interval[5] = "Sec".to_string();
            }
            Interval::Friday => {
                self.tmp_interval[5] = "Fir".to_string();
            }
            Interval::Saturday => {
                self.tmp_interval[5] = "Sat".to_string();
            }
            Interval::Sunday => {
                self.tmp_interval[5] = "Sun".to_string();
            }
            Interval::Weekday => {
                self.tmp_interval[5] = "Mon-Fir".to_string();
            }
            Interval::Seconds(x) => {
                self.tmp_interval[0] = x.to_string();
            }
            Interval::Minutes(x) => {
                self.tmp_interval[1] = x.to_string();
            }
            Interval::Hours(x) => {
                self.tmp_interval[2] = x.to_string();
            }
        }
        self
    }

    pub fn next(mut self) -> Self {
        let s = self.tmp_interval.iter().as_slice().concat();
        let s = Schedule::from_str(s.as_str()).expect("不合法");
        self.schedules.push(s);
        self.tmp_interval = vec![
            "*".to_string(),
            "*".to_string(),
            "*".to_string(),
            "*".to_string(),
            "*".to_string(),
            "*".to_string(),
            "*".to_string(),
        ];
        self
    }

    fn at(mut self, interval: Interval) -> Self {
        match interval {
            Interval::Seconds(x) => {
                self.tmp_interval[0] = x.to_string();
            }
            Interval::Minutes(x) => {
                self.tmp_interval[1] = x.to_string();
            }
            Interval::Hours(x) => {
                self.tmp_interval[2] = x.to_string();
            }
            _ => {}
        }
        self
    }

    pub fn repeat(mut self, n: usize) -> Self {
        self.repeat = Some(n);
        self
    }

    pub fn run<Params>(mut self, f: F) -> Self
    where
        Params: Clone + 'static + Debug,
        F: Handler<Params>,
    {
        self = self.next();
        self.f = Some(f);
        self
    }
}

#[async_trait]
impl<Args, F> Job for SyncJob<Args, F>
where
    F: Handler<Args> + Send + Sync + 'static + Clone + Copy,
    Args: Clone + 'static + Debug + Send + Sync,
{
    async fn start_schedule(&self, e: Extensions) {
        println!("被调度了");
        let mut han = vec![];
        for shedule in self.schedules.iter() {
            let f = self.f.clone();
            let e = e.clone();
            let shedule = shedule.clone();
            let t = tokio::spawn(async move {
                for next in shedule.upcoming(chrono::Local) {
                    let f = f.clone();
                    let e = e.clone();
                    tokio::spawn(async move {
                        f.call(e);
                    });
                    tokio::time::sleep(Duration::from_secs(60)).await;
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
            repeat: None,
            _phantom: PhantomData,
            schedules: vec![],
            tmp_interval: vec![
                "*".to_string(),
                "*".to_string(),
                "*".to_string(),
                "*".to_string(),
                "*".to_string(),
                "*".to_string(),
                "*".to_string(),
            ],
        }
    }

    // pub fn run<Params>(&self, f: F)
    // where
    //     Params: Clone + 'static + Debug,
    //     F: Handler<Params>,
    // {
    //     f.call(&self.extensions);
    // }

    // pub fn run_async<Params>(&self, f: F)
    // where
    //     Params: Clone + 'static + Debug,
    //     F: Clone + Copy + Send + Sync + 'static + AsyncHandler<Params>,
    // {
    //     let f = f.clone();
    //     let ext = self.extensions.clone();
    //     tokio::spawn(async move {
    //         loop {
    //             let f = f.clone();
    //             f.call(&ext).await;
    //             tokio::time::sleep(Duration::from_secs(10)).await;
    //         }
    //     });
    // }
}
