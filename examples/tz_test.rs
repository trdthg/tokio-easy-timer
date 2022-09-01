trait Timezone: Clone {}

#[derive(Clone)]
struct Utc;
impl Timezone for Utc {}
#[derive(Clone)]
struct Local;
impl Timezone for Local {}
#[derive(Clone)]
struct FixedOffset;
impl Timezone for FixedOffset {}

struct Scheduler<Tz = Local> {
    tz: Tz,
}

impl<Tz: Timezone> Scheduler<Tz> {
    fn with_tz(tz: Tz) -> Self {
        Self { tz }
    }

    fn add_job(&mut self, job: Builder2Job) {
        job.add_tz(self.tz.clone());
    }
}

impl Scheduler {
    fn new() -> Self {
        Self { tz: Local }
    }
}

struct Job<Tz> {
    tz: Tz,
}

impl<Tz> Job<Tz> {
    fn builder() -> JobBuilder {
        JobBuilder {}
    }
}

struct JobBuilder {}
impl JobBuilder {
    fn build() -> Builder2Job {
        Builder2Job {}
    }
}

struct Builder2Job {}

impl Builder2Job {
    fn add_tz<Tz>(&self, tz: Tz) -> Job<Tz> {
        Job { tz }
    }
}

fn main() {
    let s1 = Scheduler::new();

    for i in 0..1 {
        println!("{i}");
    }
}
