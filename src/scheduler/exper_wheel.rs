use timer_utils::extensions::Extensions;

use crate::job::SyncHandler;

struct Wheel<F> {
    pub wheel_size: usize,
    pub interval: usize,
    pub current_time: i64,
    pub buckets: Vec<Vec<F>>,
    pub next_wheel: Option<Box<Wheel<F>>>,
}

struct Task<Args> {
    delay: i64,
    inner: Box<dyn SyncHandler<Args>>,
    e: Extensions,
}

impl<Args> Task<Args> {
    pub fn new(delay: i64, f: Box<dyn SyncHandler<Args>>, e: Extensions) -> Self {
        Self { delay, inner: f, e }
    }
    pub fn run(&mut self) {
        self.inner.call(&self.e);
    }
}

struct TaskList<Args> {
    tasks: Vec<Task<Args>>,
}

impl<F> Wheel<F> {
    pub fn tick(&mut self) {
        self.current_time += 1;
        let n = self.current_time % self.interval as i64 / (self.interval / self.wheel_size) as i64;
        print!("{n} ");
        if let Some(next) = &mut self.next_wheel {
            next.tick();
        } else {
            println!();
        }
    }
}

fn main() {
    let n = chrono::Local::now().timestamp();
    let mut wheel: Wheel<i32> = Wheel {
        wheel_size: 20,
        interval: 20,
        current_time: n,
        buckets: vec![],
        next_wheel: Some(Box::new(Wheel {
            wheel_size: 20,
            interval: 20 * 20,
            current_time: n,
            buckets: vec![],
            next_wheel: None,
        })),
    };
    for i in 0..400 {
        wheel.tick();
    } 
}
