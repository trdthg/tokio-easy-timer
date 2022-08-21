use std::{future::Future, marker::PhantomData, pin::Pin, time::Duration};

use chrono::{Datelike, TimeZone};

use crate::{
    extensions::Extensions,
    interval::Interval,
    scheduler::{item::ScheduleItem, BoxedJob},
};

use super::{
    jobschedule::{JobSchedule, JobScheduleBuilder},
    AsyncHandler, Job, JobBuilder, JobId,
};

#[derive(Clone)]
pub struct AsyncJob<Args, F> {
    pub id: JobId,
    pub f: F,
    pub cancel: bool,
    pub jobschedule: JobSchedule,
    pub current_time: u64,
    pub next_schedule_index: usize,
    pub _phantom: PhantomData<Args>,
}

#[async_trait::async_trait]
impl<Args, F, Tz> Job<Tz> for AsyncJob<Args, F>
where
    F: AsyncHandler<Args> + Send + 'static + Copy,
    Args: Send + 'static + Clone,
    Tz: TimeZone + Send + Copy + 'static,
    <Tz as TimeZone>::Offset: Send,
{
    fn box_clone(&self) -> Box<(dyn Job<Tz> + Send + 'static)> {
        Box::new((*self).clone())
    }

    // fn start_schedule(&self, e: Extensions, tz: Tz) {
    //     for schedule in self.jobschedules.iter() {
    //         // spawn a task for every corn schedule
    //         let f = self.f.clone();
    //         let e = e.clone();
    //         let schedule = schedule.clone();
    //         tokio::spawn(async move {
    //             // delay
    //             let now = chrono::Local::now().with_timezone(&tz);
    //             let since = schedule.since;
    //             let wait_to = tz
    //                 .ymd(since.0, since.1, since.2)
    //                 .and_hms(since.3, since.4, since.5);
    //             let d = wait_to.timestamp() - now.timestamp();
    //             if d > 0 {
    //                 tokio::time::sleep(Duration::from_secs(d as u64)).await;
    //             }

    //             // run jobs
    //             for next in schedule.schedule.upcoming(tz) {
    //                 // Calculates the time left until the next task run
    //                 let now = chrono::Local::now().with_timezone(&tz);
    //                 let d = next.timestamp() - now.timestamp();
    //                 if d < 0 {
    //                     continue;
    //                 }
    //                 let e = e.clone();
    //                 // prepare a task for the next job
    //                 tokio::spawn(async move {
    //                     // Wait until the next job runs
    //                     tokio::time::sleep(Duration::from_secs(d as u64)).await;

    //                     // Handle repeat
    //                     for i in 0..schedule.repeat {
    //                         if schedule.is_async {
    //                             let e = e.clone();
    //                             tokio::spawn(async move {
    //                                 f.call(&e).await;
    //                             });
    //                         } else {
    //                             f.call(&e).await;
    //                         }
    //                         if schedule.interval > 0 && i < schedule.repeat - 1 {
    //                             tokio::time::sleep(Duration::from_secs(schedule.interval)).await;
    //                         }
    //                     }
    //                 });

    //                 // wait until this task run
    //                 tokio::time::sleep(Duration::from_secs(d as u64)).await;
    //             }
    //         });
    //     }
    // }

    fn current(&mut self) -> Option<ScheduleItem> {
        Some(ScheduleItem {
            id: self.id,
            time: self.current_time,
        })
    }

    fn next(&mut self, tz: Tz) -> Option<ScheduleItem> {
        if self.cancel {
            return None;
        }

        if let Some(next_time) = self
            .jobschedule
            .schedule
            .upcoming(tz)
            .take(1)
            .next()
            .and_then(|x| Some(x.timestamp() as u64))
        {
            self.current_time = next_time;
            Some(ScheduleItem {
                id: self.id,
                time: self.current_time,
            })
        } else {
            None
        }
    }

    fn next_job(&mut self, tz: Tz) -> Option<crate::scheduler::item::ScheduleJobItem<Tz>> {
        self.next(tz);
        Some(crate::scheduler::item::ScheduleJobItem {
            time: self.current_time,
            job: Box::new(self.clone()),
        })
    }

    fn run<'a: 'b, 'b>(
        &'a self,
        e: Extensions,
        tz: Tz,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>> {
        let f = self.f.clone();
        let schedule = self.jobschedule.clone();
        Box::pin(async move {
            // handle delay
            // println!("delay {}", schedule.delay);
            if schedule.delay > 0 {
                tokio::time::sleep(Duration::from_secs(schedule.delay)).await;
            }
            // handle since
            // if let Some(delay) = schedule.get_since_delay_sec(tz) {
            //     tokio::time::sleep(Duration::from_secs(delay as u64)).await;
            // }

            let e = e.clone();
            // prepare a task for the job
            tokio::spawn(async move {
                // Handle repeat
                for i in 0..schedule.repeat {
                    if schedule.is_async {
                        let e = e.clone();
                        tokio::spawn(async move {
                            f.call(&e).await;
                        });
                    } else {
                        f.call(&e).await;
                    }
                    if schedule.interval > 0 && i < schedule.repeat - 1 {
                        tokio::time::sleep(Duration::from_secs(schedule.interval)).await;
                    }
                }
            });
        })
    }

    fn get_id(&self) -> JobId {
        self.id
    }

    fn set_id(&mut self, id: JobId) {
        self.id = id;
    }

    fn start(&mut self) {
        self.cancel = false;
    }

    fn stop(&mut self) {
        self.cancel = true;
    }
}
