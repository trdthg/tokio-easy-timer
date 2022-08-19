mod extensions;
pub mod interval;
mod job;
mod scheduler;
pub use extensions::Data;
pub use job::{AsyncJobBuilder as AsyncJob, JobBuilder, SyncJobBuilder as SyncJob};
pub use job::{Job, JobId};
pub use scheduler::{BucketScheduler, HeapScheduler, Scheduler};

pub mod prelude {
    pub use crate::interval::{Interval, TimeUnits};
    pub use crate::Data;
    pub use crate::{AsyncJob, Job, JobBuilder, JobId, SyncJob};
    pub use crate::{BucketScheduler, HeapScheduler, Scheduler};
}
