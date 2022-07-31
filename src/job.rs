mod async_handler;
mod async_job;
mod jobschedule;
mod sync_handler;
mod sync_job;
pub use self::async_handler::AsyncHandler;
use self::jobschedule::JobScheduleBuilder;
pub use self::sync_handler::SyncHandler;
use crate::{extensions::Extensions, interval::Interval};
pub use async_job::{AsyncJob, AsyncJobBuilder};
use async_trait::async_trait;
use chrono::TimeZone;
pub use sync_job::{SyncJob, SyncJobBuilder};

#[async_trait]
pub trait Job<Tz>
where
    Tz: TimeZone,
{
    async fn start_schedule(&self, e: Extensions, tz: Tz);
}

pub trait JobBuilder<Args> {
    fn new() -> Self;
    fn next(&mut self) -> &mut Self;
    fn repeat(&mut self, n: u32, interval: Interval) -> &mut Self;

    fn get_cron_builder(&mut self) -> &mut JobScheduleBuilder;
    fn at(&mut self, interval: Interval) -> &mut Self {
        self.get_cron_builder().at(interval);
        self
    }

    fn since_every(&mut self, start: Interval, interval: Interval) -> &mut Self {
        self.get_cron_builder().since_every(start, interval);
        self
    }

    fn every(&mut self, interval: Interval) -> &mut Self {
        self.get_cron_builder().every(interval);
        self
    }

    fn from_to(&mut self, start: Interval, end: Interval) -> &mut Self {
        self.get_cron_builder().from_to(start, end);
        self
    }

    fn at_datetime(
        &mut self,
        year: Option<i32>,
        month: Option<u32>,
        day: Option<u32>,
        hour: Option<u32>,
        min: Option<u32>,
        sec: Option<u32>,
    ) -> &mut Self;

    fn at_time(&mut self, hour: u32, min: u32, sec: u32) -> &mut Self {
        self.at_datetime(None, None, None, Some(hour), Some(min), Some(sec));
        self
    }

    fn at_date(&mut self, year: i32, month: u32, day: u32) -> &mut Self {
        self.at_datetime(Some(year), Some(month), Some(day), None, None, None);
        self
    }

    fn since_datetime(
        &mut self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> &mut Self;

    fn since_date(&mut self, year: i32, month: u32, day: u32) -> &mut Self {
        self.since_datetime(year, month, day, 0, 0, 0);
        self
    }

    fn since_time(&mut self, hour: u32, min: u32, sec: u32) -> &mut Self {
        self.since_datetime(0, 1, 1, hour, min, sec);
        self
    }
}
