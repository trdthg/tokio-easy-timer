#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Interval {
    /// The next multiple of `n` seconds since the start of the Unix epoch
    Seconds(u32),
    /// The next multiple of `n` minutes since the start of the day
    Minutes(u32),
    /// The next multiple of `n` hours since the start of the day
    Hours(u32),
    /// The next multiple of `n` days since the start of the start of the era
    Days(u32),
    /// The next multiple of `n` week since the start of the start of the era
    Weeks(u32),
    /// Every Monday
    Monday,
    /// Every Tuesday
    Tuesday,
    /// Every Wednesday
    Wednesday,
    /// Every Thursday
    Thursday,
    /// Every Friday
    Friday,
    /// Every Saturday
    Saturday,
    /// Every Sunday
    Sunday,
    /// Every weekday (Monday through Friday)
    Weekday,
}

/// ```
pub trait TimeUnits: Sized {
    fn seconds(self) -> Interval;
    fn minutes(self) -> Interval;
    fn hours(self) -> Interval;
    fn days(self) -> Interval;
    fn weeks(self) -> Interval;
    fn second(self) -> Interval {
        self.seconds()
    }
    fn minute(self) -> Interval {
        self.minutes()
    }
    fn hour(self) -> Interval {
        self.hours()
    }
    fn day(self) -> Interval {
        self.days()
    }
    fn week(self) -> Interval {
        self.weeks()
    }
}

impl TimeUnits for u32 {
    fn seconds(self) -> Interval {
        Interval::Seconds(self)
    }
    fn minutes(self) -> Interval {
        Interval::Minutes(self)
    }
    fn hours(self) -> Interval {
        Interval::Hours(self)
    }
    fn days(self) -> Interval {
        Interval::Days(self)
    }
    fn weeks(self) -> Interval {
        Interval::Weeks(self)
    }
}

pub struct YearCron {
    sec: Vec<u32>,
    min: Vec<u32>,
    hour: Vec<u32>,
    day: Vec<u32>,
    month: Vec<u32>,
    week: Vec<u32>,
    year: Vec<Option<u32>>,
}
pub struct WeekCron {
    sec: Vec<u32>,
    min: Vec<u32>,
    hour: Vec<u32>,
    day: Vec<u32>,
    month: Vec<u32>,
    week: Vec<u32>,
    year: Vec<Option<u32>>,
}
pub struct MonthCron {
    sec: Vec<u32>,
    min: Vec<u32>,
    hour: Vec<u32>,
    day: Vec<u32>,
    month: Vec<u32>,
    week: Vec<u32>,
    year: Vec<Option<u32>>,
}
pub struct DayCron {
    sec: Vec<u32>,
    min: Vec<u32>,
    hour: Vec<u32>,
    day: Vec<u32>,
    month: Vec<u32>,
    week: Vec<u32>,
    year: Vec<Option<u32>>,
}
pub struct HourCron {
    sec: Vec<u32>,
    min: Vec<u32>,
    hour: Vec<u32>,
    day: Vec<u32>,
    month: Vec<u32>,
    week: Vec<u32>,
    year: Vec<Option<u32>>,
}
pub struct MinCron {
    sec: Vec<u32>,
    min: Vec<u32>,
    hour: Vec<u32>,
    day: Vec<u32>,
    month: Vec<u32>,
    week: Vec<u32>,
    year: Vec<Option<u32>>,
}
pub struct SecCron {
    sec: Vec<u32>,
    min: Vec<u32>,
    hour: Vec<u32>,
    day: Vec<u32>,
    month: Vec<u32>,
    week: Vec<u32>,
    year: Vec<Option<u32>>,
}
impl YearCron {
    pub fn new() -> Self {
        Self {
            sec: vec![],
            min: vec![],
            hour: vec![],
            day: vec![],
            month: vec![],
            week: vec![],
            year: vec![],
        }
    }
    pub fn year(mut self, year: u32) -> WeekCron {
        self.year.push(Some(year));
        WeekCron {
            sec: self.sec,
            min: self.min,
            hour: self.hour,
            day: self.day,
            month: self.month,
            week: self.week,
            year: self.year,
        }
    }
}

impl WeekCron {
    pub fn new() -> Self {
        Self {
            sec: vec![],
            min: vec![],
            hour: vec![],
            day: vec![],
            month: vec![],
            week: vec![],
            year: vec![],
        }
    }
    pub fn week(mut self, week: u32) -> MonthCron {
        self.week.push(week);
        MonthCron {
            sec: self.sec,
            min: self.min,
            hour: self.hour,
            day: self.day,
            month: self.month,
            week: self.week,
            year: self.year,
        }
    }
}

impl MonthCron {
    pub fn new() -> Self {
        Self {
            sec: vec![],
            min: vec![],
            hour: vec![],
            day: vec![],
            month: vec![],
            week: vec![],
            year: vec![],
        }
    }
    pub fn day(mut self, day: u32) -> DayCron {
        self.day.push(day);
        DayCron {
            sec: self.sec,
            min: self.min,
            hour: self.hour,
            day: self.day,
            month: self.month,
            week: self.week,
            year: self.year,
        }
    }
}

impl HourCron {
    pub fn new() -> Self {
        Self {
            sec: vec![],
            min: vec![],
            hour: vec![],
            day: vec![],
            month: vec![],
            week: vec![],
            year: vec![],
        }
    }
    pub fn week(mut self, hour: u32) -> HourCron {
        self.hour.push(hour);
        HourCron {
            sec: self.sec,
            min: self.min,
            hour: self.hour,
            day: self.day,
            month: self.month,
            week: self.week,
            year: self.year,
        }
    }
}

impl MinCron {
    pub fn new() -> Self {
        Self {
            sec: vec![],
            min: vec![],
            hour: vec![],
            day: vec![],
            month: vec![],
            week: vec![],
            year: vec![],
        }
    }
    pub fn min(mut self, min: u32) -> SecCron {
        self.min.push(min);
        SecCron {
            sec: self.sec,
            min: self.min,
            hour: self.hour,
            day: self.day,
            month: self.month,
            week: self.week,
            year: self.year,
        }
    }
}

impl SecCron {
    pub fn new() -> Self {
        Self {
            sec: vec![],
            min: vec![],
            hour: vec![],
            day: vec![],
            month: vec![],
            week: vec![],
            year: vec![],
        }
    }
    pub fn sec(mut self, sec: u32) {
        self.sec.push(sec);
    }
}
