use std::{
    collections::{HashMap, LinkedList, VecDeque},
    sync::Arc,
    time::Duration,
};

use chrono::TimeZone;
use parking_lot::Mutex;
use timer_utils::extensions::Extensions;

use crate::{scheduler, BaseJob, Job, JobId, Scheduler};

use super::{item::ScheduleItem, BoxedJob};

pub struct TimingWheel {
    tick_duration: u64, // 每个格子的时间跨度
    wheel_size: usize,  // 时间轮的格数
    interval: u64,      // 当前时间轮的总体时间跨度：tick * size

    // start_time: u64,   // 开始时间：构造时间轮的时间，上层的开始时间是下层的当前时间
    current_time: u64, // 当前时间：current_time = start_ms - (start_ms % tick_ms)

    buckets: Buckets, // 任务列表
}

pub struct Buckets {
    inner: Vec<LinkedList<ScheduleItem>>,
}

impl Buckets {
    pub fn new(size: usize) -> Self {
        let cycle = {
            let mut res = vec![];
            for _ in 0..size {
                res.push(LinkedList::new());
            }
            res
        };
        Self { inner: cycle }
    }
}

impl TimingWheel {
    pub fn new(wheel_size: usize, tick_duration: u64, current_time: u64) -> Self {
        Self {
            tick_duration,
            wheel_size,
            interval: wheel_size as u64 * tick_duration,
            // start_time: 0,
            current_time,
            buckets: Buckets::new(wheel_size),
        }
    }

    pub fn advance_to(&mut self, time: u64) -> bool {
        // timeMs 超过了当前 bucket 的时间范
        // if time >= self.current_time + self.tick_duration {
        //     // 修改当前时间，即原先的第一个桶已经失效
        //     self.current_time = time - (time % self.tick_duration)
        // }
        self.current_time = time;
        self.current_time % self.tick_duration == 0
    }
}

pub struct TimingWheelScheduler<Tz = chrono::Local>
where
    Tz: TimeZone + Clone + Sync + Send + Copy + 'static,
    <Tz as TimeZone>::Offset: Send + Sync,
{
    extensions: Extensions,
    tz: Tz,
    max_id: usize,
    current_time: u64,
    jobs: HashMap<JobId, BoxedJob<Tz>>,
    wheels: Arc<Mutex<Vec<TimingWheel>>>,
    exec_queue: Arc<Mutex<VecDeque<ScheduleItem>>>, // todo 延时队列
    delay_queue: Arc<Mutex<VecDeque<ScheduleItem>>>, // todo 延时队列
}

impl TimingWheelScheduler {
    pub fn new() -> Self {
        // tick_duration: u64, sizes: Vec<usize>
        let (tick_duration, sizes) = (1, vec![60, 60, 24]);
        assert!(sizes.len() > 0);
        let current_time = chrono::Utc::now().timestamp();
        let max_id = 0;
        let first_wheel = TimingWheel::new(sizes[0], tick_duration, current_time as u64);
        let wheels = {
            let mut wheels: Vec<TimingWheel> = vec![];
            wheels.push(first_wheel);
            for (i, size) in sizes[1..].iter().enumerate() {
                let wheel = TimingWheel::new(*size, wheels[i].interval, current_time as u64);
                wheels.push(wheel);
            }
            wheels
        };
        Self {
            max_id,
            jobs: HashMap::new(),
            wheels: Arc::new(Mutex::new(wheels)),
            exec_queue: Arc::new(Mutex::new(VecDeque::new())),
            delay_queue: Arc::new(Mutex::new(VecDeque::new())),
            current_time: current_time as u64,
            tz: chrono::Local,
            extensions: Extensions::default(),
        }
    }
}
enum Layer {
    Exec,
    Normal(usize, usize),
    Delay,
}

impl<Tz> TimingWheelScheduler<Tz>
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
    pub fn with_tz(tick_duration: u64, sizes: Vec<usize>, tz: Tz) -> Self {
        assert!(sizes.len() > 0);
        let max_id = 0;
        let current_time = chrono::Utc::now().timestamp();
        let first_wheel = TimingWheel::new(sizes[0], tick_duration, current_time as u64);
        let wheels = {
            let mut wheels: Vec<TimingWheel> = vec![];
            wheels.push(first_wheel);
            for (i, size) in sizes[1..].iter().enumerate() {
                let wheel = TimingWheel::new(*size, wheels[i].interval, current_time as u64);
                wheels.push(wheel);
            }
            wheels
        };
        Self {
            max_id,
            jobs: HashMap::new(),
            wheels: Arc::new(Mutex::new(wheels)),
            exec_queue: Arc::new(Mutex::new(VecDeque::new())),
            delay_queue: Arc::new(Mutex::new(VecDeque::new())),
            current_time: current_time as u64,
            tz,
            extensions: Extensions::default(),
        }
    }

    fn get_layer(wheels: &Vec<TimingWheel>, item: &ScheduleItem) -> Layer {
        for i in 0..wheels.len() {
            let expiration = item.time;
            let wheel = &wheels[i];
            if expiration < (wheel.current_time + wheel.tick_duration) as u64 {
                // 任务已经过期
                return Layer::Exec;
            } else if expiration < (wheel.current_time + wheel.interval) as u64 {
                // 任务可以插入当前时间轮
                let virtual_id = expiration / wheel.tick_duration as u64;
                return Layer::Normal(i, virtual_id as usize % wheel.wheel_size);
            } else {
                // 任务必须插入下一层时间轮
                continue;
            }
        }
        return Layer::Delay;
    }

    // #[cfg(test)]
    fn print_wheel(&self) {
        println!("--------------------------------");
        println!("层\t时间\t\t桶编号\t数量\t具体");
        for wheel_index in 0..3 {
            let mut b = false;
            let wheel = &self.wheels.lock()[wheel_index];
            for (i, link) in wheel.buckets.inner.iter().enumerate() {
                if link.len() > 0 {
                    print!(
                        "{}\t{}\t{}\t{}\t",
                        wheel_index,
                        wheel.current_time,
                        i,
                        link.len()
                    );
                    // for item in link {
                    //     print!(" {:?} ", *item.time.lock().unwrap());
                    // }
                    println!();
                    b = true;
                }
            }
            if !b {
                print!("{}\t{}\t{}\t{}\t", wheel_index, wheel.current_time, 0, 0);
                println!();
            }
        }

        println!();
    }

    fn tick(&mut self) {
        let Self {
            wheels,
            current_time,
            ..
        } = self;

        *current_time += 1;
        let mut res = LinkedList::new();
        let len = wheels.lock().len();
        for i in 0..len {
            let wheel = &mut wheels.lock()[i];
            let end = wheel.advance_to(*current_time);

            let bucket_index = wheel.current_time as usize % wheel.interval as usize
                / wheel.tick_duration as usize;
            let bucket = &mut wheel.buckets.inner[bucket_index];

            println!(
                "第 {i} 指针指向第 {} 个桶，里面有 {} 个元素 队列：{} {}",
                bucket_index,
                bucket.len(),
                self.exec_queue.lock().len(),
                self.delay_queue.lock().len()
            );

            if bucket.len() > 0 {
                res.append(bucket);
            }
            // if !end {
            // break;
            // }
        }

        // add to exec_queue or add to lower level theel
        // for link in res.iter_mut() {
        while let Some(item) = res.pop_back() {
            // 一些到期需要运行，一些需要降级
            Self::add_to_schedule(self.wheels.clone(), self.exec_queue.clone(), item);
        }
        // }
    }

    fn add_to_schedule(
        wheels: Arc<Mutex<Vec<TimingWheel>>>,
        exec_queue: Arc<Mutex<VecDeque<ScheduleItem>>>,
        item: ScheduleItem,
    ) {
        let layer = Self::get_layer(&wheels.lock(), &item);
        match layer {
            Layer::Normal(layer, pos) => {
                let wheel = &mut wheels.lock()[layer];
                wheel.buckets.inner[pos].push_back(item);
            }
            Layer::Exec => {
                exec_queue.lock().push_back(item);
            }
            Layer::Delay => {
                exec_queue.lock().push_back(item);
            }
        }
    }
}

#[async_trait::async_trait]
impl<Tz> Scheduler<Tz> for TimingWheelScheduler<Tz>
where
    Tz: TimeZone + Clone + Sync + Send + Copy + 'static,
    <Tz as TimeZone>::Offset: Send + Sync,
{
    fn get_tz(&self) -> Tz {
        self.tz.clone()
    }

    fn add_job(&mut self, mut job: BoxedJob<Tz>) -> &mut dyn Scheduler<Tz> {
        let id = JobId(self.max_id);
        job.set_id(id);
        let item = job.next();
        if let Some(item) = item {
            Self::add_to_schedule(self.wheels.clone(), self.exec_queue.clone(), item);
        }
        self.jobs.insert(id, job);
        self.max_id += 1;
        self
    }

    async fn run(&mut self) {
        // self.print_wheel();
        // 执行任务
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let wheels_clone = self.wheels.clone();
        let exec_queue_clone = self.exec_queue.clone();
        tokio::spawn(async move {
            while let Some(next) = rx.recv().await {
                let wheels_clone = wheels_clone.clone();
                let exec_queue_clone = exec_queue_clone.clone();
                Self::add_to_schedule(wheels_clone, exec_queue_clone, next);
            }
        });

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            self.tick();
            // self.print_wheel();
            let Self { jobs, .. } = self;
            let tx = tx.clone();
            println!("{}", self.exec_queue.lock().len());
            while let Some(item) = self.exec_queue.lock().pop_back() {
                if let Some(job) = jobs.get_mut(&item.id) {
                    let mut job = job.box_clone();
                    let e = self.extensions.clone();
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        // // prepare the next job
                        if let Some(next) = job.next() {
                            tx.send(next).expect("任务放置失败");
                        }
                        drop(tx);
                        // run
                        let fut = job.run(e);
                        fut.await;
                    });
                }
            }
        }
    }
}
