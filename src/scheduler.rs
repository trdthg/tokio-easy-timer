use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    sync::Arc,
};

use chrono::{FixedOffset, TimeZone};

use crate::job::{AsyncHandler, Job};
use crate::{
    extensions::{DebugAny, Extensions},
    job::SyncJob,
};

pub type BoxedJob<Tz> = Arc<dyn Job<Tz> + Send + Sync + 'static>;
pub struct Scheduler<Tz = chrono::Local>
where
    Tz: chrono::TimeZone,
{
    jobs: Vec<BoxedJob<Tz>>,
    tz: Tz,
    extensions: Extensions,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            extensions: Extensions::default(),
            jobs: vec![],
            tz: chrono::Local,
        }
    }

    pub fn with_tz<Tzz: chrono::TimeZone>(tz: Tzz) -> Scheduler<Tzz> {
        Scheduler {
            extensions: Extensions::default(),
            jobs: vec![],
            tz,
        }
    }

    // pub fn add(mut self, job: BoxedJob) -> Self {
    //     self.jobs.push(job);
    //     self
    // }
}

impl<Tz> Scheduler<Tz>
where
    Tz: chrono::TimeZone + Send + 'static,
{
    pub fn add_ext<T>(&self, ext: T)
    where
        T: DebugAny + 'static + Send + Sync,
    {
        self.extensions.insert(ext);
    }

    pub fn add(&mut self, job: BoxedJob<Tz>) -> &mut Scheduler<Tz> {
        self.jobs.push(job);
        self
    }

    // pub fn add<Args, F>(&mut self, job: SyncJob<Args, F>) -> &mut Scheduler<Tz>
    // where
    //     Args: Clone + 'static + Debug + Send + Sync,
    //     F: AsyncHandler<Args> + Copy + Send + Sync + 'static,
    // {
    //     let job = Arc::new(SyncJob {
    //         f: job.f,
    //         _phantom: PhantomData,
    //         jobschedules: job.jobschedules,
    //     });
    //     self.jobs.push(job);
    //     self
    // }

    pub async fn start(&self) {
        let mut hans = vec![];
        for job in self.jobs.iter() {
            let e = self.extensions.clone();
            let tz = self.tz.clone();
            let job = job.to_owned();
            let t = tokio::spawn(async move {
                job.start_schedule(e, tz).await;
            });
            hans.push(t);
        }
        for t in hans {
            if let Err(e) = t.await {}
        }
    }
}
