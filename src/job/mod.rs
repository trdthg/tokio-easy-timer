mod async_handler;
mod async_job;
mod base_job;
mod jobschedule;
mod sync_handler;
mod sync_job;
pub use self::async_handler::AsyncHandler;
pub use self::async_job::AsyncJob;
pub use self::jobschedule::JobScheduleBuilder;
pub use self::sync_handler::SyncHandler;
pub use self::sync_job::SyncJob;
use crate::{
    extensions::Extensions,
    interval::Interval,
    prelude::TimeUnits,
    scheduler::item::{ScheduleItem, ScheduleJobItem},
};
use async_trait::async_trait;
pub use base_job::BaseJobBuilder;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JobId(pub usize);

#[async_trait]
pub trait Job<Tz> {
    fn box_clone(&self) -> Box<dyn Job<Tz> + Send>;

    fn next(&mut self, tz: Tz) -> Option<ScheduleItem>;

    fn next_job(&mut self, tz: Tz) -> Option<ScheduleJobItem<Tz>>;

    // /// Start spawn jobs
    async fn run(&self, e: Extensions, tz: Tz);

    fn get_id(&self) -> JobId;
    fn set_id(&mut self, id: JobId);
}

pub trait JobBuilder {
    /// Constructs a new job builder
    fn new() -> Self;

    /// Specify another time the task will run
    fn and(&mut self) -> &mut Self;

    /// The task will run n times in sequence. You need to specify the number and the interval between tasks.
    fn repeat_seq(&mut self, n: u32, interval: Interval) -> &mut Self;

    /// Spawn n tasks at the same time. You need to specify the number and the interval between tasks.
    fn repeat_async(&mut self, n: u32, interval: Interval) -> &mut Self;

    /// Usually not use it directly, use `at_*` and `since_*` is better
    fn get_mut_cron_builder(&mut self) -> &mut JobScheduleBuilder;

    /// Specify a specific run time, equivalent to cron 'n'
    fn at(&mut self, interval: Interval) -> &mut Self {
        self.get_mut_cron_builder().at(interval);
        self
    }

    /// Specify a specific run time, equivalent to the corn expression 'm/n'
    fn since_every(&mut self, start: Interval, interval: Interval) -> &mut Self {
        self.get_mut_cron_builder().since_every(start, interval);
        self
    }

    /// Specify a specific run time, equivalent to the corn expression '0/n'
    fn every(&mut self, interval: Interval) -> &mut Self {
        self.get_mut_cron_builder().every(interval);
        self
    }

    /// Specify a specific run time, equivalent to the corn expression 'm-n'
    fn from_to(&mut self, start: Interval, end: Interval) -> &mut Self {
        self.get_mut_cron_builder().from_to(start, end);
        self
    }

    /// Specify a specific run time, the same as `at`
    fn at_datetime(
        &mut self,
        year: Option<i32>,
        month: Option<u32>,
        day: Option<u32>,
        hour: Option<u32>,
        min: Option<u32>,
        sec: Option<u32>,
    ) -> &mut Self {
        year.map(|x| self.at((x as u32).year()));
        month.map(|x| self.at((x as u32).month()));
        day.map(|x| self.at((x as u32).day()));
        hour.map(|x| self.at((x as u32).hour()));
        min.map(|x| self.at((x as u32).minute()));
        sec.map(|x| self.at((x as u32).second()));
        self
    }

    /// Specify a specific run time, the same as `at`
    fn at_time(&mut self, hour: u32, min: u32, sec: u32) -> &mut Self {
        self.at_datetime(None, None, None, Some(hour), Some(min), Some(sec));
        self
    }

    /// Specify a specific run time, the same as `at`
    fn at_date(&mut self, year: i32, month: u32, day: u32) -> &mut Self {
        self.at_datetime(Some(year), Some(month), Some(day), None, None, None);
        self
    }

    fn get_mut_since(&mut self) -> &mut (Option<(i32, u32, u32)>, Option<(u32, u32, u32)>);

    /// Specify the datetime after which the task will start, the same as `since`
    fn since_datetime(
        &mut self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> &mut Self {
        if let (Some(ymd), Some(hms)) = self.get_mut_since() {
            *ymd = (year, month, day);
            *hms = (hour, min, sec);
        }
        self
    }

    /// Specify the date after which the task will start, the same as `since`
    fn since_date(&mut self, year: i32, month: u32, day: u32) -> &mut Self {
        if let (Some(ymd), _) = self.get_mut_since() {
            *ymd = (year, month, day);
        }
        self
    }

    /// Specify the time after which the task will start, the same as `since`
    fn since_time(&mut self, hour: u32, min: u32, sec: u32) -> &mut Self {
        if let (_, Some(hms)) = self.get_mut_since() {
            *hms = (hour, min, sec);
        }
        self
    }

    /// Specify when the task will start after, like `since`
    fn after(&mut self, delay: u64) -> &mut Self {
        self.get_mut_cron_builder().delay(delay);
        self
    }
}
