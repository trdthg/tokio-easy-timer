use std::{cmp::Reverse, collections::BinaryHeap, sync::Arc};

use parking_lot::Mutex;

use super::{
    bucket::ItemBucket,
    item::{Item, ScheduleItem},
};

#[derive(Debug, Clone)]
struct HeapTimeBuckets {
    inner: Arc<Mutex<BinaryHeap<Reverse<ItemBucket>>>>,
}

impl Buckets for HeapTimeBuckets {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }
    fn add(&self, item: ScheduleItem) {
        for i in self.inner.lock().iter() {
            if i.0.get_time() == item.time {
                i.0.push(item);
                return;
            }
        }
        let bucket = ItemBucket::new(item.get_time());
        bucket.push(item);
        self.inner.lock().push(Reverse(bucket));
    }
}
impl HeapTimeBuckets {
    fn pop(&self) -> Option<(u64, Vec<ScheduleItem>)> {
        self.inner
            .lock()
            .pop()
            .map(|x| (x.0.get_time(), x.0.get_items()))
    }
}

use chrono::TimeZone;
use std::{collections::HashMap, future::Future, pin::Pin};

use crate::{extensions::Extensions, JobId};

use super::{bucket::Buckets, BoxedJob, Scheduler};

pub struct BucketScheduler<Tz = chrono::Local>
where
    Tz: chrono::TimeZone,
{
    max_id: usize,
    jobs: HashMap<JobId, BoxedJob<Tz>>,
    heap: HeapTimeBuckets,
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
impl<Tz> Scheduler<Tz> for BucketScheduler<Tz>
where
    Tz: TimeZone + Clone + Sync + Send + Copy + 'static,
    <Tz as TimeZone>::Offset: Send + Sync,
{
    /// Start the timer, block the current thread.
    async fn run_pending(&mut self) {
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
            let (time, items) = item.unwrap();
            let t = time;
            let n = chrono::Utc::now().timestamp() as u64;
            for item in items {
                // find the associate job
                if let Some(job) = self.get(&item.id) {
                    let mut job = job.box_clone();

                    let tz = self.tz.clone();
                    let e = self.extensions.clone();
                    let tx = tx.clone();

                    // spawn a task
                    tokio::spawn(async move {
                        // calcute the delay time, and wait
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
        }
        // std::future::pending::<()>().await;
    }

    fn add(&mut self, mut job: BoxedJob<Tz>) -> &mut dyn Scheduler<Tz> {
        self.max_id += 1;
        job.set_id(crate::JobId(self.max_id));
        if let Some(item) = job.next(self.tz) {
            self.heap.add(item);
        }
        self.jobs.insert(job.get_id(), job);
        self
    }

    fn get(&self, id: &JobId) -> Option<&BoxedJob<Tz>> {
        self.jobs.get(id)
    }

    fn get_mut(&mut self, id: &JobId) -> Option<&mut BoxedJob<Tz>> {
        self.jobs.get_mut(id)
    }

    fn remove(&mut self, id: &JobId) {
        self.jobs.remove(id);
    }
}

impl BucketScheduler {
    /// ## Constructs a new scheduler
    ///
    /// the default timezone is chrono::Local, if you want a specified timezone, use `Scheduler::with_tz()` instead.
    ///
    /// ### Example
    ///
    /// ```rust
    /// let s = Scheduler::new();
    /// ```
    pub fn new() -> BucketScheduler {
        BucketScheduler {
            extensions: Extensions::default(),
            jobs: HashMap::new(),
            tz: chrono::Local,
            heap: Buckets::new(),
            max_id: 0,
        }
    }

    /// if you want a specified timezone instead of the mathine timezone `chrono::Local`, use this
    pub fn with_tz<Tz: chrono::TimeZone>(tz: Tz) -> BucketScheduler<Tz> {
        BucketScheduler {
            extensions: Extensions::default(),
            jobs: HashMap::new(),
            tz,
            heap: Buckets::new(),
            max_id: 0,
        }
    }
}

impl<Tz> BucketScheduler<Tz>
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
