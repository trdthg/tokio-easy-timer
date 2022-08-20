pub mod interval;
mod job;
pub mod scheduler;
pub use extensions::Data;
pub use job::{AsyncJobBuilder as AsyncJob, JobBuilder, SyncJobBuilder as SyncJob};
pub use job::{Job, JobId};
pub use scheduler::{HeapScheduler, Scheduler};
pub use timer_utils::extensions;

pub mod prelude {
    pub use crate::interval::{Interval, TimeUnits};
    pub use crate::scheduler;
    pub use crate::Data;
    pub use crate::Scheduler;
    pub use crate::{AsyncJob, Job, JobBuilder, JobId, SyncJob};
}
