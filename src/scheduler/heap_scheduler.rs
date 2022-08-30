use chrono::TimeZone;
use parking_lot::Mutex;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    sync::Arc,
};

use crate::{extensions::Extensions, job::SyncJobBuilder, JobId};

use super::{bucket::Buckets, item::ScheduleItem, BoxedJob, Scheduler};

pub struct HeapScheduler<Tz = chrono::Local>
where
    Tz: chrono::TimeZone,
{
    max_id: usize,
    jobs: HashMap<JobId, BoxedJob<Tz>>,
    heap: HeapItemBuckets,
    tz: Tz,
    extensions: Extensions,
}

#[derive(Debug, Clone)]
struct HeapItemBuckets {
    inner: Arc<Mutex<BinaryHeap<Reverse<ScheduleItem>>>>,
}

impl Buckets for HeapItemBuckets {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }
    fn add(&self, item: ScheduleItem) {
        self.inner.lock().push(Reverse(item));
    }
}
impl HeapItemBuckets {
    fn pop(&self) -> Option<ScheduleItem> {
        self.inner.lock().pop().and_then(|x| Some(x.0))
    }
}

#[async_trait::async_trait]
impl<Tz> Scheduler<Tz> for HeapScheduler<Tz>
where
    Tz: TimeZone + Clone + Sync + Send + Copy + 'static,
    <Tz as TimeZone>::Offset: Send + Sync,
{
    fn get_tz(&self) -> Tz {
        self.tz.clone()
    }
    /// Start the timer, block the current thread.
    async fn run(&mut self) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let heap = self.heap.clone();
        tokio::spawn(async move {
            while let Some(next) = rx.recv().await {
                heap.add(next);
            }
        });

        loop {
            // take the fist expired item
            let item = self.heap.pop();

            // if no item then wait 1 sec
            if item.is_none() {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
            let item = item.unwrap();

            // find the associate job
            if let Some(job) = self.jobs.get(&item.id) {
                let mut job = job.box_clone();

                let tz = self.tz.clone();
                let e = self.extensions.clone();
                let tx = tx.clone();

                // spawn a task
                tokio::spawn(async move {
                    // calcute the delay time, and wait
                    let t = item.time;
                    // let n = chrono::Utc::now().timestamp() as u64;
                    let n = timer_cacher::get_cached_timestamp();
                    if t > n {
                        tokio::time::sleep(std::time::Duration::from_secs(t - n)).await;
                    }

                    // prepare the next and send it out
                    let next = job.next(tz);
                    if let Some(next) = next {
                        if let Err(e) = tx.send(next) {
                            println!("send next job failed: {}", e);
                        }
                    }

                    // run
                    let fut = job.run(e, tz);
                    fut.await;
                });
            }
        }
        // std::future::pending::<()>().await;
    }

    fn add_job(&mut self, mut job: BoxedJob<Tz>) -> &mut dyn Scheduler<Tz> {
        self.max_id += 1;
        job.set_id(crate::JobId(self.max_id));
        if let Some(item) = job.next(self.tz) {
            self.heap.add(item);
        }
        self.jobs.insert(job.get_id(), job);
        self
    }
}

impl HeapScheduler {
    /// ## Constructs a new scheduler
    ///
    /// the default timezone is chrono::Local, if you want a specified timezone, use `Scheduler::with_tz()` instead.
    ///
    /// ### Example
    ///
    /// ```rust
    /// let s = Scheduler::new();
    /// ```
    pub fn new() -> HeapScheduler {
        HeapScheduler {
            extensions: Extensions::default(),
            jobs: HashMap::new(),
            tz: chrono::Local,
            heap: Buckets::new(),
            max_id: 0,
        }
    }

    /// if you want a specified timezone instead of the mathine timezone `chrono::Local`, use this
    pub fn with_tz<Tz: chrono::TimeZone>(tz: Tz) -> HeapScheduler<Tz> {
        HeapScheduler {
            extensions: Extensions::default(),
            jobs: HashMap::new(),
            tz,
            heap: Buckets::new(),
            max_id: 0,
        }
    }
}

impl<Tz> HeapScheduler<Tz>
where
    Tz: TimeZone + Clone + Sync + Send + Copy + 'static,
    <Tz as TimeZone>::Offset: Send + Sync,
{
    /// add a type to the map, you can use it later in the task closer
    pub fn add_ext<T>(&self, ext: T)
    where
        T: 'static + Send + Sync,
    {
        self.extensions.insert(ext);
    }

    // pub fn add<Args, F>(&mut self, job: AsyncJob<Args, F>) -> &mut Scheduler<Tz>
    // where
    //     Args: Clone + 'static + Send + Sync,
    //     F: AsyncHandler<Args> + Copy + Send + Sync + 'static,
    // {
    //     let job = Box::new(job);
    //     self.jobs.push(job);
    //     self
    // }
}
