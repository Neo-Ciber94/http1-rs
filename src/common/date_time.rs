use std::{fmt::Display, time::SystemTime};

const SECONDS_IN_MILLIS: u128 = 1000;
const MINUTES_IN_MILLIS: u128 = SECONDS_IN_MILLIS * 60;
const HOURS_IN_MILLIS: u128 = MINUTES_IN_MILLIS * 60;
const DAYS_IN_MILLIS: u128 = HOURS_IN_MILLIS * 24;
const YEAR_IN_MILLIS: u128 = DAYS_IN_MILLIS * 365;
const LEAP_YEAR_IN_MILLIS: u128 = DAYS_IN_MILLIS * 366;

// Days per month in a common year (January to December)
const DAYS_IN_MONTH_COMMON: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

// Days per month in a leap year
const DAYS_IN_MONTH_LEAP: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

//
const YEAR_EPOCH: u32 = 1970;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Month {
    January = 0,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Month {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Month::January => "January",
            Month::February => "February",
            Month::March => "March",
            Month::April => "April",
            Month::May => "May",
            Month::June => "June",
            Month::July => "July",
            Month::August => "August",
            Month::September => "September",
            Month::October => "October",
            Month::November => "November",
            Month::December => "December",
        }
    }
}

impl Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug)]
pub struct InvalidMonthIndex;

impl TryFrom<u8> for Month {
    type Error = InvalidMonthIndex;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Month::January),
            1 => Ok(Month::February),
            2 => Ok(Month::March),
            3 => Ok(Month::April),
            4 => Ok(Month::May),
            5 => Ok(Month::June),
            6 => Ok(Month::July),
            7 => Ok(Month::August),
            8 => Ok(Month::September),
            9 => Ok(Month::October),
            10 => Ok(Month::November),
            11 => Ok(Month::December),
            _ => Err(InvalidMonthIndex),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime(u128);

impl DateTime {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn now_utc() -> Self {
        let system_time = SystemTime::now();
        let duration = system_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Failed to get system time");
        let ms = duration.as_millis();
        Self::with_millis(ms)
    }

    pub fn with_millis(ms: u128) -> Self {
        Self(ms)
    }

    pub fn with_yymmdd(year: u32, month: Month, day: u8) -> Self {
        Builder::new().year(year).month(month).day(day).build()
    }

    pub fn as_millis(&self) -> u128 {
        self.0
    }

    pub fn as_days(&self) -> u128 {
        self.as_millis() / DAYS_IN_MILLIS
    }

    pub fn is_leap_year(&self) -> bool {
        is_leap_year(self.year())
    }

    pub fn year(&self) -> u64 {
        let mut remaining_days = self.as_days() as u64;
        let mut year = YEAR_EPOCH as u64;

        while remaining_days >= days_in_year(year) {
            remaining_days -= days_in_year(year);
            year += 1;
        }

        year
    }

    pub fn month(&self) -> Month {
        let millis = self.as_millis();
        let total_millis_until_year = millis_until_year(self.year());
        let mut remaining_ms = millis - total_millis_until_year;

        let days_in_month = if is_leap_year(self.year()) {
            &DAYS_IN_MONTH_LEAP
        } else {
            &DAYS_IN_MONTH_COMMON
        };

        // Calculate which month corresponds to the remaining milliseconds
        for (month_idx, &days) in days_in_month.iter().enumerate() {
            let month_ms = days as u128 * DAYS_IN_MILLIS;

            if remaining_ms < month_ms {
                return Month::try_from(month_idx as u8).expect("Failed to get month from index");
            }

            remaining_ms -= month_ms;
        }

        Month::December
    }

    pub fn day_of_month(&self) -> u8 {
        let millis = self.as_millis();
        let total_millis_until_year = millis_until_year(self.year());
        let mut remaining_ms = millis - total_millis_until_year;

        let days_in_month = if is_leap_year(self.year()) {
            &DAYS_IN_MONTH_LEAP
        } else {
            &DAYS_IN_MONTH_COMMON
        };

        // Subtract milliseconds for each month until we get to the current month
        for &days in days_in_month.iter() {
            let month_ms = days as u128 * DAYS_IN_MILLIS;

            if remaining_ms < month_ms {
                break;
            }
            remaining_ms -= month_ms;
        }

        // Calculate the day of the month based on the remaining milliseconds in the current month
        let day = remaining_ms / DAYS_IN_MILLIS;

        (day + 1) as u8
    }

    pub fn hours(&self) -> u8 {
        let remaining_ms_in_day = self.remaining_ms_in_day();
        (remaining_ms_in_day / HOURS_IN_MILLIS) as u8
    }

    pub fn minutes(&self) -> u8 {
        let remaining_ms_in_day = self.remaining_ms_in_day();
        let remaining_ms_in_hour = remaining_ms_in_day % HOURS_IN_MILLIS;
        (remaining_ms_in_hour / MINUTES_IN_MILLIS) as u8
    }

    pub fn secs(&self) -> u8 {
        let remaining_ms_in_day = self.remaining_ms_in_day();
        let remaining_ms_in_hour = remaining_ms_in_day % HOURS_IN_MILLIS;
        let remaining_ms_in_minute = remaining_ms_in_hour % MINUTES_IN_MILLIS;
        (remaining_ms_in_minute / SECONDS_IN_MILLIS) as u8
    }

    pub fn millis(&self) -> u16 {
        let remaining_ms_in_day = self.remaining_ms_in_day();
        let remaining_ms_in_hour = remaining_ms_in_day % HOURS_IN_MILLIS;
        let remaining_ms_in_minute = remaining_ms_in_hour % MINUTES_IN_MILLIS;
        let remaining_ms_in_second = remaining_ms_in_minute % SECONDS_IN_MILLIS;
        remaining_ms_in_second as u16
    }

    fn remaining_ms_in_day(&self) -> u128 {
        let millis = self.as_millis();
        let days = self.as_days();
        let millis_in_full_days = days * DAYS_IN_MILLIS;
        millis - millis_in_full_days
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DateTimeInfo {
    pub year: u64,
    pub month: u8,
    pub day: u8,
    pub hours: u8,
    pub minutes: u8,
    pub secs: u8,
    pub millis: u16,
}

impl DateTime {
    fn fmt_to_iso_8601_string(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let year: u64 = self.year();
        let day = self.day_of_month();
        let month = self.month().as_u8() + 1;
        let hour = self.hours();
        let mins = self.minutes();
        let secs = self.secs();
        let millis = self.millis();

        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{hour:02}:{mins:02}:{secs:02}:{millis:03}Z"
        )
    }

    pub fn to_iso_8601_string(&self) -> String {
        self.map(
            |DateTimeInfo {
                 year,
                 month,
                 day,
                 hours,
                 minutes,
                 secs,
                 millis,
             }| {
                format!(
                    "{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{secs:02}.{millis:03}Z"
                )
            },
        )
    }

    pub fn map<F: FnOnce(&DateTimeInfo) -> String>(&self, f: F) -> String {
        let year = self.year();
        let day = self.day_of_month();
        let month = self.month().as_u8() + 1;
        let hours = self.hours();
        let minutes = self.minutes();
        let secs = self.secs();
        let millis = self.millis();

        f(&DateTimeInfo {
            year,
            day,
            month,
            hours,
            minutes,
            secs,
            millis,
        })
    }
}

impl Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_to_iso_8601_string(f)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Builder {
    year: u32,
    month: Month,
    day: u8,
    hours: u8,
    minutes: u8,
    secs: u8,
    millis: u16,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            year: 1970,
            month: Month::January,
            day: 0,
            hours: 0,
            minutes: 0,
            secs: 0,
            millis: 0,
        }
    }

    pub fn year(mut self, year: u32) -> Self {
        self.year = year;
        self
    }

    pub fn month(mut self, month: Month) -> Self {
        self.month = month;
        self
    }

    pub fn day(mut self, day: u8) -> Self {
        assert!(day > 0, "day should be greater than 0");
        self.day = day;
        self
    }

    pub fn hours(mut self, hours: u8) -> Self {
        self.hours = hours;
        self
    }

    pub fn minutes(mut self, minutes: u8) -> Self {
        self.minutes = minutes;
        self
    }

    pub fn secs(mut self, secs: u8) -> Self {
        self.secs = secs;
        self
    }

    pub fn millis(mut self, millis: u16) -> Self {
        self.millis = millis;
        self
    }

    pub fn build(self) -> DateTime {
        let Self {
            year,
            month,
            day,
            hours,
            minutes,
            secs,
            millis,
        } = self;

        let mut ms: u128 = 0;

        // Add milliseconds for full years since the epoch
        for y in YEAR_EPOCH..year {
            ms += millis_in_year(y as u64) as u128;
        }

        // Add milliseconds for the months in the current year
        let days_in_month = if is_leap_year(year as u64) {
            &DAYS_IN_MONTH_LEAP
        } else {
            &DAYS_IN_MONTH_COMMON
        };

        for days_in_month in days_in_month.iter().cloned().take(month as usize) {
            ms += days_in_month as u128 * DAYS_IN_MILLIS;
        }

        // Add milliseconds for the remaining days, hours, seconds, and millis
        ms += (day as u128 - 1) * DAYS_IN_MILLIS;
        ms += hours as u128 * HOURS_IN_MILLIS;
        ms += minutes as u128 * MINUTES_IN_MILLIS;
        ms += secs as u128 * SECONDS_IN_MILLIS;
        ms += millis as u128;

        DateTime::with_millis(ms)
    }
}

fn millis_until_year(year: u64) -> u128 {
    let years_ms = (year as u128 - YEAR_EPOCH as u128) * YEAR_IN_MILLIS;
    let year_days = (0..(year as u128 - YEAR_EPOCH as u128))
        .filter(|y| is_leap_year(YEAR_EPOCH as u64 + *y as u64))
        .count() as u128;

    years_ms + (year_days * DAYS_IN_MILLIS)
}

fn days_in_year(year: u64) -> u64 {
    if is_leap_year(year) {
        366
    } else {
        365
    }
}

fn millis_in_year(year: u64) -> u64 {
    if is_leap_year(year) {
        LEAP_YEAR_IN_MILLIS as u64
    } else {
        YEAR_IN_MILLIS as u64
    }
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::common::date_time::Month;

    use super::DateTime;

    #[test]
    fn should_create_date_with_builder() {
        let dt = DateTime::builder()
            .year(2024)
            .month(Month::September)
            .day(20)
            .hours(18)
            .minutes(30)
            .secs(45)
            .millis(190)
            .build();

        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), Month::September);
        assert_eq!(dt.day_of_month(), 20);
        assert_eq!(dt.hours(), 18);
        assert_eq!(dt.minutes(), 30);
        assert_eq!(dt.secs(), 45);
        assert_eq!(dt.millis(), 190);
    }

    #[test]
    fn should_display_date_as_iso_format() {
        let dt = DateTime::builder()
            .year(2025)
            .month(Month::January)
            .day(3)
            .hours(16)
            .minutes(15)
            .secs(42)
            .millis(555)
            .build();

        assert_eq!(dt.to_iso_8601_string(), "2025-01-03T16:15:42.555Z")
    }
}
