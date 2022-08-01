mod extensions;
pub mod interval;
mod job;
mod scheduler;

pub use extensions::Data;
pub use interval::{Interval, TimeUnits};
pub use job::{AsyncJobBuilder as AsyncJob, JobBuilder, SyncJobBuilder as SyncJob};
pub use scheduler::Scheduler;

pub mod prelude {
    pub use crate::Data;
    pub use crate::Scheduler;
    pub use crate::{AsyncJob, JobBuilder, SyncJob};
    pub use crate::{Interval, TimeUnits};
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    #[test]
    fn test() {
        let s = "0 0 0 * * 7,1 *";
        let s = cron::Schedule::from_str(s);
        for i in s.unwrap().upcoming(chrono::Local).take(100) {
            println!("{}", i);
        }
    }
}
