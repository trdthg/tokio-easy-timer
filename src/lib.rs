mod extensions;
pub mod interval;
mod job;
mod scheduler;

pub use extensions::Data;
pub use job::{AsyncJobBuilder as AsyncJob, JobBuilder, SyncJobBuilder as SyncJob};
pub use scheduler::Scheduler;

pub mod prelude {
    pub use crate::interval::{Interval, TimeUnits};
    pub use crate::Data;
    pub use crate::Scheduler;
    pub use crate::{AsyncJob, JobBuilder, SyncJob};
}
