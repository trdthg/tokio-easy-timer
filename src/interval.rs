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
    /// The next multiple of `n` months since the start of the start of the era
    Months(u32),
    /// The next multiple of `n` weeks since the start of the start of the era
    Weeks(u32),
    /// The next multiple of `n` years since the start of the start of the era
    Years(u32),
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

impl Interval {
    pub(crate) fn to_sec(self) -> u64 {
        match self {
            Interval::Seconds(x) => x as u64,
            Interval::Minutes(x) => x as u64 * 60,
            Interval::Hours(x) => x as u64 * 3600,
            Interval::Days(x) => x as u64 * 3600 * 24,
            Interval::Weeks(x) => x as u64 * 3600 * 24 * 7,
            _ => unimplemented!(),
        }
    }
}

pub trait TimeUnits: Sized {
    fn seconds(self) -> Interval;
    fn minutes(self) -> Interval;
    fn hours(self) -> Interval;
    fn days(self) -> Interval;
    fn months(self) -> Interval;
    fn weeks(self) -> Interval;
    fn years(self) -> Interval;

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
    fn month(self) -> Interval {
        self.months()
    }
    fn year(self) -> Interval {
        self.years()
    }
    fn week(self) -> Interval {
        self.weeks()
    }
}

impl TimeUnits for u32 {
    /// Turn u32 to Interval::Seconds(u32)
    ///
    /// # Example
    /// ```rust
    /// let a = 1.seconds();
    /// assert_eq!(a, Interval::Seconds(1));
    /// ```
    fn seconds(self) -> Interval {
        assert!(self < 60);
        Interval::Seconds(self)
    }

    /// Turn u32 to Interval::Minutes(u32)
    ///
    /// # Example
    /// ```rust
    /// let a = 1.minutes();
    /// assert_eq!(a, Interval::Minutes(1));
    /// ```
    fn minutes(self) -> Interval {
        assert!(self < 60);
        Interval::Minutes(self)
    }

    /// Turn u32 to Interval::Hours(u32)
    ///
    /// # Example
    /// ```rust
    /// let a = 1.hours();
    /// assert_eq!(a, Interval::Hours(1));
    /// ```
    fn hours(self) -> Interval {
        assert!(self < 24);
        Interval::Hours(self)
    }

    /// Turn u32 to Interval::Days(u32)
    ///
    /// # Example
    /// ```rust
    /// let a = 1.days();
    /// assert_eq!(a, Interval::Days(1));
    /// ```
    fn days(self) -> Interval {
        assert!(self <= 31);
        Interval::Days(self)
    }

    /// Turn u32 to Interval::Months(u32)
    ///
    /// # Example
    /// ```rust
    /// let a = 1.months();
    /// assert_eq!(a, Interval::Months(1));
    /// ```
    fn months(self) -> Interval {
        assert!(self <= 12);
        Interval::Months(self)
    }

    /// Turn u32 to Interval::Weeks(u32)
    ///
    /// # Example
    /// ```rust
    /// let a = 1.weeks();
    /// assert_eq!(a, Interval::Weeks(1));
    /// ```
    fn weeks(self) -> Interval {
        assert!(self <= 7);
        Interval::Weeks(self)
    }

    /// Turn u32 to Interval::Years(u32)
    ///
    /// # Example
    /// ```rust
    /// let a = 1.years();
    /// assert_eq!(a, Interval::Years(1));
    /// ```
    fn years(self) -> Interval {
        assert!(self <= 2100);
        Interval::Years(self)
    }
}
