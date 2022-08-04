use chrono::TimeZone;

use crate::extensions::Extensions;
use crate::job::Job;

pub type BoxedJob<Tz> = Box<dyn Job<Tz> + Send + 'static>;
pub struct Scheduler<Tz = chrono::Local>
where
    Tz: chrono::TimeZone,
{
    jobs: Vec<BoxedJob<Tz>>,
    tz: Tz,
    extensions: Extensions,
}

impl Scheduler {
    /// ## Constructs a new scheduler
    ///
    /// the default timezone is chrono::Local, if you want a specified timezone, use `Scheduler::with_tz()` instead.
    ///
    /// ### Example
    ///
    /// ```rust
    /// let s = Scheduler::new();
    /// ```
    pub fn new() -> Scheduler {
        Scheduler {
            extensions: Extensions::default(),
            jobs: vec![],
            tz: chrono::Local,
        }
    }

    /// if you want a specified timezone instead of the mathine timezone `chrono::Local`, use this
    pub fn with_tz<Tz: chrono::TimeZone>(tz: Tz) -> Scheduler<Tz> {
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

    /// add a new task to the scheduler, you must privide something that implements `Job` trait.
    pub fn add(&mut self, job: BoxedJob<Tz>) -> &mut Scheduler<Tz> {
        self.jobs.push(job);
        self
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

    async fn start_spawn(&self) -> &Self {
        for job in self.jobs.iter() {
            let e = self.extensions.clone();
            let tz = self.tz.clone();
            {
                let job = job.box_clone();
                tokio::spawn(async move {
                    let job = job;
                    job.start_schedule(e, tz);
                });
            }
        }
        self
    }

    /// Start the timer.
    pub async fn run(&self) -> &Self {
        self.start_spawn().await
    }

    /// Start the timer, block the current thread.
    pub async fn run_pending(&self) {
        self.start_spawn().await;
        std::future::pending::<()>().await;
    }
}
