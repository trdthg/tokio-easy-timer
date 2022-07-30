use crate::extensions::{DebugAny, Extensions};
use crate::job::Job;

pub type BoxedJob = Box<dyn Job + Send + Sync + 'static>;
pub struct Scheduler {
    jobs: Vec<BoxedJob>,
    extensions: Extensions,
}
impl Scheduler {
    pub fn new() -> Self {
        Self {
            extensions: Extensions::default(),
            jobs: vec![],
        }
    }

    pub fn add_ext<T>(&self, ext: T)
    where
        T: DebugAny + 'static + Send + Sync,
    {
        self.extensions.insert(ext);
    }

    pub fn add(mut self, job: BoxedJob) -> Self {
        self.jobs.push(job);
        self
    }

    pub async fn start(self) {
        let mut hans = vec![];
        for job in self.jobs.into_iter() {
            let e = self.extensions.clone();
            let t = tokio::spawn(async move {
                job.start_schedule(e).await;
            });
            hans.push(t);
        }
        for t in hans {
            if let Err(e) = t.await {

            }
        }
    }
}
