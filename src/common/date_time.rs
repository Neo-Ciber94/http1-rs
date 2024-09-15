use std::time::SystemTime;

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime(u128);

impl DateTime {
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

    pub fn with_yymmdd(year: u32, month: u8, day: u8) -> Self {
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
        let mut remaining_days: u64 = self.as_days() as u64;
        let mut year = YEAR_EPOCH as u64;

        while remaining_days >= days_in_year(year) {
            remaining_days -= days_in_year(year);
            year += 1;
        }

        year
    }

    pub fn month(&self) -> u32 {
        let millis = self.as_millis();
        let total_millis_until_year = millis_until_year(self.year());
        let mut remaining_ms = millis - total_millis_until_year;

        let days_in_month = if is_leap_year(self.year() as u64) {
            &DAYS_IN_MONTH_LEAP
        } else {
            &DAYS_IN_MONTH_COMMON
        };

        // Calculate which month corresponds to the remaining milliseconds
        for (month_idx, &days) in days_in_month.iter().enumerate() {
            let month_ms = days as u128 * DAYS_IN_MILLIS;

            if remaining_ms < month_ms {
                return month_idx as u32;
            }

            remaining_ms -= month_ms;
        }

        11
    }

    pub fn day_of_month(&self) -> u8 {
        let millis = self.as_millis();
        let total_millis_until_year = millis_until_year(self.year());
        let mut remaining_ms = millis - total_millis_until_year;

        let days_in_month = if is_leap_year(self.year() as u64) {
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
        todo!()
    }

    pub fn minutes(&self) -> u8 {
        todo!()
    }

    pub fn secs(&self) -> u8 {
        todo!()
    }

    pub fn millis(&self) -> u16 {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Builder {
    year: u32,
    month: u8,
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
            month: 0,
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

    pub fn month(mut self, month: u8) -> Self {
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

        for m in 0..(month + 1) as usize {
            ms += days_in_month[m] as u128 * DAYS_IN_MILLIS;
        }

        // Add milliseconds for the remaining days, hours, seconds, and millis
        ms += (day as u128) * DAYS_IN_MILLIS;
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

#[cfg(test)]
mod tests {
    use super::DateTime;

    #[test]
    fn should_get_date_year() {
        let now = DateTime::now_utc();
        assert_eq!(now.year(), 2024);
        assert_eq!(now.month(), 8);
        assert_eq!(now.day_of_month(), 15);
    }
}
