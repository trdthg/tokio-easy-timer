use crate::interval::Interval;
use chrono::{Datelike, TimeZone};
use cron::Schedule;
use std::str::FromStr;

#[derive(Clone)]
pub struct JobSchedule {
    pub since: (Option<(i32, u32, u32)>, Option<(u32, u32, u32)>),
    pub delay: u64,
    pub schedule: Schedule,
    pub is_async: bool,
    pub repeat: u32,
    pub interval: u64,
}

impl JobSchedule {
    pub fn get_since_delay_sec<Tz: TimeZone>(&self, tz: Tz) -> Option<i64> {
        if self.since != (None, None) {
            let (ymd, hms) = self.since;
            let hms = hms.unwrap_or((0, 0, 0));
            let now = chrono::Utc::now().with_timezone(&tz);
            let ymd = ymd.unwrap_or((now.year(), now.month(), now.day()));
            let wait_to = tz.ymd(ymd.0, ymd.1, ymd.2).and_hms(hms.0, hms.1, hms.2);
            let d = wait_to.timestamp() - now.timestamp();
            if d > 0 {
                return Some(d);
            }
        }
        None
    }
}

pub struct JobScheduleBuilder {
    pub since: (Option<(i32, u32, u32)>, Option<(u32, u32, u32)>),
    pub delay: u64,
    pub cron: Vec<Option<String>>,
    pub is_async: bool,
    pub repeat: u32,
    pub interval: u64,
}

impl JobScheduleBuilder {
    pub fn new() -> Self {
        Self {
            since: (None, None),
            cron: vec![None, None, None, None, None, None, None],
            repeat: 1,
            interval: 1,
            is_async: false,
            delay: 0,
        }
    }

    pub fn delay(&mut self, delay: u64) -> &mut Self {
        self.delay += delay;
        self
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

        let s = Schedule::from_str(s.as_str())
            .expect(&format!("cron expression is not valid: {}", s.as_str()));
        JobSchedule {
            schedule: s,
            repeat: self.repeat as u32,
            interval: self.interval,
            since: self.since,
            is_async: self.is_async,
            delay: self.delay,
        }
    }
}

macro_rules! every_start {
    ($( {$Varient: ident, $Index: expr} ),* | ($WeekIndex:expr) | $({$WeekVarient: ident, $I: expr} ),* ) => {

        impl JobScheduleBuilder {
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
                                s.push_str(format!(",*/{}", x).as_str());
                            } else {
                                self.cron[$Index] = Some(format!("*/{}", x));
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

#[cfg(test)]
mod test {
    use std::str::FromStr;

    #[test]
    fn test() {
        let cron = cron::Schedule::from_str("0 0 0 * * * *").unwrap();
        let a = cron.upcoming(chrono::Local).take(1).next();
        let b = cron.upcoming(chrono::Local).take(1).next();
        assert_eq!(a, b);
    }

    #[test]
    fn test_since_every() {
        let cron = cron::Schedule::from_str("0 2/5 0 * * * *").unwrap();
        for t in cron.upcoming(chrono::Local).take(5).into_iter() {
            println!("{:?}", t);
        }
    }
}
