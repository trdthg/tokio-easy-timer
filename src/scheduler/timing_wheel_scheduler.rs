use std::{
    collections::{HashMap, LinkedList, VecDeque},
    time::Duration,
};

use chrono::TimeZone;
use timer_utils::extensions::Extensions;

use crate::{JobId, Scheduler};

use super::{task::ScheduleItem, BoxedJob};

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
    wheels: Vec<TimingWheel>,
    exec_queue: VecDeque<ScheduleItem>,  // todo 延时队列
    delay_queue: VecDeque<ScheduleItem>, // todo 延时队列
}

impl Default for TimingWheelScheduler {
    fn default() -> Self {
        // tick_duration: u64, sizes: Vec<usize>
        let (tick_duration, sizes) = (1, vec![60, 60, 24]);
        assert!(!sizes.is_empty());
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
            wheels,
            exec_queue: VecDeque::new(),
            delay_queue: VecDeque::new(),
            current_time: current_time as u64,
            tz: chrono::Local,
            extensions: Extensions::default(),
        }
    }
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
        assert!(!sizes.is_empty());
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
            wheels,
            exec_queue: VecDeque::new(),
            delay_queue: VecDeque::new(),
            current_time: current_time as u64,
            tz,
            extensions: Extensions::default(),
        }
    }

    fn get_layer(&self, item: &ScheduleItem) -> Option<(usize, usize)> {
        for i in 0..self.wheels.len() {
            let expiration = item.time;
            let wheel = &self.wheels[i];
            // 2 + 22
            if expiration < (wheel.current_time + wheel.tick_duration) {
                // 任务已经过期
                return None;
            } else if expiration < (wheel.current_time + wheel.interval) {
                // 任务可以插入当前时间轮
                let virtual_id = expiration / wheel.tick_duration;
                // let bucket = &wheel.buckets[(virtual_id as usize % wheel.wheel_size)];
                return Some((i, virtual_id as usize % wheel.wheel_size));
                // // 设置过期时间，这里也取整了，即可以被 tickMs 整除
                // if (bucket.setExpiration(virtualId * tickMs)) { // 仅在新的过期时间和之前的不同才返回 true
                //     // 由于进行了取整，同一个 bucket 所有节点的过期时间都相同，因此仅在 bucket 的第一个节点加入时才会进入此 if 块
                //     // 因此保证了每个桶只会被加入一次到 queue 中，queue 存放所有包含定时任务节点的 bucket
                //     // 借助 DelayQueue 来检测 bucket 是否过期，bucket 时遍历即可取出所有节点
                //     queue.offer(bucket)
                // }
                // true
            } else {
                // 任务必须插入下一层时间轮
                continue;
            }
        }
        None
    }

    // #[cfg(test)]
    fn print_wheel(&self) {
        println!("--------------------------------");
        println!("层\t时间\t桶编号\t数量\t具体");
        for wheel_index in 0..3 {
            let mut b = false;
            for (i, link) in self.wheels[wheel_index].buckets.inner.iter().enumerate() {
                if !link.is_empty() {
                    print!(
                        "{}\t{}\t{}\t{}\t",
                        wheel_index,
                        self.wheels[wheel_index].current_time,
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
                print!(
                    "{}\t{}\t{}\t{}\t",
                    wheel_index, self.wheels[wheel_index].current_time, 0, 0
                );
                println!();
            }
        }
        println!();
    }

    fn add_to_exec_queue(&mut self, item: ScheduleItem) {
        if let Some((layer, pos)) = self.get_layer(&item) {
            let wheel = &mut self.wheels[layer];
            wheel.buckets.inner[pos].push_back(item);
        } else {
            // Push to Exec Queue
            self.exec_queue.push_back(item);
        }
    }

    fn tick(&mut self) {
        let Self {
            wheels,
            current_time,
            ..
        } = self;

        *current_time += 1;
        let mut res = vec![];
        for wheel in &mut wheels.iter_mut() {
            let end = wheel.advance_to(*current_time);

            let bucket_index = wheel.current_time as usize % wheel.wheel_size;
            let bucket = &mut wheel.buckets.inner[bucket_index];

            println!(
                "当前指针指向第 {} 个桶，里面有 {} 个元素 {} {}",
                bucket_index,
                bucket.len(),
                wheel.current_time,
                wheel.wheel_size
            );
            if bucket.is_empty() {
                let mut link = LinkedList::new();
                link.append(bucket);
                res.push(link);
            }

            if !end {
                break;
            }
        }

        for link in res.iter_mut() {
            while let Some(item) = link.pop_back() {
                // 一些到期需要运行，一些需要降级
                self.add_to_exec_queue(item);
            }
        }
    }

    fn add_to_schedule(&mut self, item: ScheduleItem) {
        // println!("{} {}", item.time, self.current_time);
        if let Some((layer, pos)) = self.get_layer(&item) {
            let wheel = &mut self.wheels[layer];
            wheel.buckets.inner[pos].push_back(item);
            // println!("任务被放置在第 {} 层，第 {} 个桶", layer, pos);
        } else {
            // println!("oh no");
            // Push to Delay Queue
            self.delay_queue.push_back(item);
        }
    }

    #[cfg(test)]
    async fn run_debug(&mut self) {
        // 执行任务
        let mut n = 100;
        while n > 0 {
            // n -= 1;
            self.tick();

            self.print_wheel();

            let Self { jobs, .. } = self;
            let mut nexts = vec![];
            while let Some(item) = self.exec_queue.pop_back() {
                if let Some(job) = jobs.get_mut(&item.id) {
                    let next = job.next();

                    let e = self.extensions.clone();
                    let job = job.box_clone();
                    tokio::spawn(async move {
                        let fut = job.run(e);
                        fut.await;
                    })
                    .await;

                    // 添加新任务
                    if let Some(next) = next {
                        nexts.push(next);
                    }
                }
            }
            for next in nexts {
                self.add_to_schedule(next);
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
        self.tz
    }

    fn add_job(&mut self, mut job: BoxedJob<Tz>) -> &mut dyn Scheduler<Tz> {
        let id = JobId(self.max_id);
        job.set_id(id);
        let item = job.next();
        if let Some(item) = item {
            self.add_to_schedule(item);
        }
        self.jobs.insert(id, job);
        self.max_id += 1;
        self
    }

    async fn run(&mut self) {
        self.print_wheel();
        // 执行任务
        loop {
            std::thread::sleep(Duration::from_secs(1));
            // println!("{}", chrono::Local::now().time());

            self.tick();

            // println!("V2: {}", self.current_time);
            // 表盘转动，拿出最新的任务并运行
            // self.print_wheel();

            let Self { jobs, .. } = self;
            let mut nexts = vec![];
            while let Some(item) = self.exec_queue.pop_back() {
                if let Some(job) = jobs.get_mut(&item.id) {
                    let next = job.next();

                    let e = self.extensions.clone();
                    let job = job.box_clone();
                    tokio::spawn(async move {
                        let fut = job.run(e);
                        fut.await;
                    });
                    // 添加新任务
                    if let Some(next) = next {
                        nexts.push(next);
                    }
                }
            }
            for next in nexts {
                self.add_to_schedule(next);
            }
        }
    }
}

#[cfg(test)]
mod test {

    use std::{
        sync::{Arc, Mutex},
        time::{Duration, Instant},
    };

    use timer_utils::extensions::Data;

    use crate::{
        interval::{Interval, TimeUnits},
        scheduler::TimingWheelScheduler,
        BaseJob, HeapScheduler, Job, JobBuilder, Scheduler,
    };

    struct Config {
        id: i32,
    }

    #[tokio::test]
    async fn a() {
        let mut scheduler = TimingWheelScheduler::new();
        let config = Arc::new(Mutex::new(Config { id: 0 }));
        let c2 = config.clone();
        scheduler.add_ext(config);

        let builder = BaseJob::new().every(10.seconds()).run_async(
            |config: Data<Arc<Mutex<Config>>>| async move {
                if let Ok(mut config) = config.lock() {
                    config.id += 1;
                }
            },
        );

        for i in 0..200000 {
            scheduler.add_asyncbuilder(builder.clone());
        }

        let start_time = std::time::Instant::now();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(1));
            println!(
                "[{}] check: {}",
                start_time.elapsed().as_secs(),
                c2.lock().unwrap().id
            );
        });

        scheduler.add_ext(std::time::Instant::now());
        // scheduler.add_syncbuilder(
        //     BaseJob::new()
        //         .since_every(5.seconds(), 10.seconds())
        //         .run_sync(|config: Data<Arc<Mutex<Config>>>, start: Data<Instant>| {
        //             if let Ok(config) = config.lock() {
        //                 println!("[{}] check: {}", start.elapsed().as_secs(), config.id);
        //             }
        //         }),
        // );

        println!("{}", "开始");

        scheduler.run_debug().await;
        std::future::pending::<()>().await;
    }
}
