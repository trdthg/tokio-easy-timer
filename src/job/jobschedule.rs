use crate::interval::Interval;
use chrono::{DateTime, TimeZone};
use cron::Schedule;
use std::str::FromStr;

#[derive(Clone)]
pub struct JobSchedule {
    pub since: (i32, u32, u32, u32, u32, u32),
    pub schedule: Schedule,
    pub is_async: bool,
    pub repeat: u32,
    pub interval: u64,
}

pub struct JobScheduleBuilder<Tz = chrono::Local>
where
    Tz: TimeZone,
{
    pub since: (i32, u32, u32, u32, u32, u32),
    pub after: DateTime<Tz>,
    pub cron: Vec<Option<String>>,
    pub is_async: bool,
    pub repeat: u32,
    pub interval: u64,
}

impl JobScheduleBuilder {
    pub fn new() -> Self {
        Self {
            since: (0, 1, 1, 0, 0, 0),
            cron: vec![None, None, None, None, None, None, None],
            after: chrono::Local::now(),
            repeat: 1,
            interval: 1,
            is_async: false,
        }
    }
}

impl<Tz> JobScheduleBuilder<Tz>
where
    Tz: TimeZone,
{
    pub fn with_tz(tz: DateTime<Tz>) -> Self {
        Self {
            since: (0, 1, 1, 0, 0, 0),
            cron: vec![None, None, None, None, None, None, None],
            after: tz,
            repeat: 1,
            interval: 1,
            is_async: false,
        }
    }

    pub fn build(&mut self) -> JobSchedule {
        for i in 0..6 {
            if self.cron[i].is_some() && self.cron[i + 1].is_none() {
                self.cron[i + 1] = Some("*".to_string())
            }
        }
        let s = self
            .cron
            .iter()
            .enumerate()
            .map(|(i, x)| {
                // x.as_deref().unwrap_or("0")
                x.as_deref().unwrap_or_else(|| match i {
                    0 | 1 | 2 => "0",
                    _ => "*",
                })
            })
            .collect::<Vec<&str>>()
            .join(" ");

        let s = Schedule::from_str(s.as_str()).expect("cron expression is not valid");
        JobSchedule {
            schedule: s,
            repeat: self.repeat as u32,
            interval: self.interval,
            since: self.since,
            is_async: false,
        }
    }
}

macro_rules! every_start {
    ($( {$Varient: ident, $Index: expr} ),* | ($WeekIndex:expr) | $({$WeekVarient: ident, $I: expr} ),* ) => {
        impl<Tz> JobScheduleBuilder<Tz>

where
Tz: TimeZone,
        {
            pub fn at(&mut self, interval: Interval) -> &mut Self {
                match interval {
                    $(
                        Interval::$Varient(x) => {
                            if let Some(s) = &mut self.cron[$Index] {
                                s.push_str(format!(",{}", x).as_str());
                            } else {
                                self.cron[$Index] = Some(format!("{}", x));
                            }
                        }
                    )*
                    $(
                        Interval::$WeekVarient => {
                            if let Some(s) = &mut self.cron[$WeekIndex] {
                                s.push_str(format!(",{}", $I).as_str());
                            } else {
                                self.cron[$WeekIndex] = Some(format!("{}", $I));
                            }
                        }
                    )*
                    Interval::Weekday => {
                        self.cron[$WeekIndex] = Some("2-6".to_string());
                    }
                }
                self
            }

            pub fn since_every(&mut self, start: Interval, interval: Interval) -> &mut Self {
                match (start, interval) {
                    $(
                        (Interval::$Varient(start), Interval::$Varient(interval)) => {
                            if let Some(s) = &mut self.cron[$Index] {
                                s.push_str(format!(",{}/{}", start, interval).as_str());
                            } else {
                                self.cron[$Index] = Some(format!("{}/{}", start, interval));
                            }
                        }
                    )*
                    _ => unimplemented!(),
                }
                self
            }

            pub fn every(&mut self, interval: Interval) -> &mut Self {
                match interval {
                    $(
                        Interval::$Varient(x) => {
                            if let Some(s) = &mut self.cron[$Index] {
                                s.push_str(format!(",0/{}", x).as_str());
                            } else {
                                self.cron[$Index] = Some(format!("0/{}", x));
                            }
                        }
                    )*
                    $(
                        Interval::$WeekVarient => {
                            let week = format!("{}", $I);
                            if let Some(s) = &mut self.cron[$WeekIndex] {
                                s.push_str(format!(",{}", week).as_str());
                            } else {
                                self.cron[$WeekIndex] = Some(week);
                            }
                        }
                    )*
                    Interval::Weekday => {
                        self.cron[$WeekIndex] = Some("2-6".to_string());
                    }
                }
                self
            }

            pub fn from_to(&mut self, start: Interval, end: Interval) -> &mut Self {
                match (start, end) {
                    $(
                        (Interval::$Varient(start), Interval::$Varient(end)) => {
                            if let Some(s) = &mut self.cron[$Index] {
                                s.push_str(format!(",{}-{}", start, end).as_str());
                            } else {
                                self.cron[$Index] = Some(format!("{}-{}", start, end));
                            }
                        }
                    )*
                    _ => unimplemented!(),
                }
                self
            }
        }
    };
}

every_start!({Seconds, 0}, {Minutes, 1}, {Hours, 2}, {Days, 3}, {Months, 4}, {Weeks, 5}, {Years, 6} | (5) | { Sunday, 1 }, { Monday, 2},  { Tuesday, 3 }, { Wednesday, 4 }, { Thursday, 5 }, { Friday, 6 }, { Saturday, 7 });
