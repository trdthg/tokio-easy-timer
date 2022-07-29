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
