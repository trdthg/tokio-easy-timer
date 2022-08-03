mod async_handler;
mod async_job;
mod jobschedule;
mod sync_handler;
mod sync_job;
use self::async_handler::AsyncHandler;
use self::jobschedule::JobScheduleBuilder;
use self::sync_handler::SyncHandler;
use crate::{extensions::Extensions, interval::Interval, prelude::TimeUnits};
pub use async_job::{AsyncJob, AsyncJobBuilder};
use async_trait::async_trait;
use chrono::TimeZone;
pub use sync_job::{SyncJob, SyncJobBuilder};

#[async_trait]
pub trait Job<Tz>
where
    Tz: TimeZone,
{
    /// Start spawn jobs
    async fn start_schedule(&self, e: Extensions, tz: Tz);
}

pub trait JobBuilder<Args> {
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

    fn get_mut_since(&mut self) -> &mut (i32, u32, u32, u32, u32, u32);

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
        *(self.get_mut_since()) = (year, month, day, hour, min, sec);
        self
    }

    /// Specify the date after which the task will start, the same as `since`
    fn since_date(&mut self, year: i32, month: u32, day: u32) -> &mut Self {
        self.get_mut_since().0 = year;
        self.get_mut_since().1 = month;
        self.get_mut_since().2 = day;
        self
    }

    /// Specify the time after which the task will start, the same as `since`
    fn since_time(&mut self, hour: u32, min: u32, sec: u32) -> &mut Self {
        self.get_mut_since().3 = hour;
        self.get_mut_since().4 = min;
        self.get_mut_since().5 = sec;
        self
    }

    /// Specify when the task will start after, like `since`
    fn after(&mut self, delay: u64) -> &mut Self {
        self.get_mut_cron_builder().add_delay(delay);
        self
    }
}
