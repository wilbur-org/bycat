use core::fmt::{self, Write};

// fn is_leap(n: i16) -> bool {
//     // Check if n is divisible by 4
//     if n % 4 == 0 {
//         // If it's divisible by 100, it should also be
//         // divisible by 400 to be a leap year
//         if n % 100 == 0 {
//             return n % 400 == 0;
//         }
//         true
//     } else {
//         false
//     }
// }

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime {
    date: Date,
    time: Time,
}

impl DateTime {
    pub fn new(date: Date, time: Time) -> DateTime {
        DateTime { date, time }
    }

    pub fn date(&self) -> Date {
        self.date
    }

    pub fn time(&self) -> Time {
        self.time
    }
}

impl fmt::Debug for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.date.fmt(f)?;
        f.write_char('T')?;
        self.time.fmt(f)
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.date.fmt(f)?;
        f.write_char(' ')?;
        self.time.fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    repr: u32,
}

impl Date {
    pub fn new(day: u8, month: u8, year: i16) -> Date {
        if day > 31 || day < 1 || month > 12 || month < 1 {
            panic!("Date")
        }

        let repr = ((day as u32) << 24) | ((month as u32) << 16) | ((year as u32) << 0);
        Date { repr }
    }

    pub fn year(&self) -> i16 {
        ((self.repr >> 0) & 0x7FFF) as i16
    }

    pub fn day(&self) -> u8 {
        ((self.repr >> 24) & 0xFF) as u8
    }

    pub fn month(&self) -> u8 {
        ((self.repr >> 16) & 0xFF) as u8
    }
}

impl fmt::Debug for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use core::fmt::Write;

        let year = self.year();

        if (0..=9999).contains(&year) {
            write_hundreds(f, (year / 100) as u8)?;
            write_hundreds(f, (year % 100) as u8)?;
        } else {
            // ISO 8601 requires the explicit sign for out-of-range years
            write!(f, "{year:+05}")?;
        }

        f.write_char('-')?;
        write_hundreds(f, self.month() as u8)?;
        f.write_char('-')?;
        write_hundreds(f, self.day() as u8)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(&self, f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    pub secs: u32,
    pub frac: u32,
}

impl Time {
    pub fn from_hm(hour: u32, mins: u32) -> Time {
        Self::from_hms(hour, mins, 0)
    }

    pub fn from_hms(hour: u32, mins: u32, secs: u32) -> Time {
        Self::from_hmsn(hour, mins, secs, 0)
    }

    pub fn from_hmsn(hour: u32, mins: u32, secs: u32, nano: u32) -> Time {
        let secs = hour * 3600 + mins * 60 + secs;
        Time { secs, frac: nano }
    }

    pub fn hour(&self) -> u32 {
        self.hms().0
    }

    pub fn minute(&self) -> u32 {
        self.hms().1
    }

    pub fn second(&self) -> u32 {
        self.hms().2
    }

    pub fn nanosecond(&self) -> u32 {
        self.frac
    }

    pub(crate) fn hms(&self) -> (u32, u32, u32) {
        let sec = self.secs % 60;
        let mins = self.secs / 60;
        let min = mins % 60;
        let hour = mins / 60;
        (hour, min, sec)
    }
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (hour, min, sec) = self.hms();
        let (sec, nano) = if self.frac >= 1_000_000_000 {
            (sec + 1, self.frac - 1_000_000_000)
        } else {
            (sec, self.frac)
        };

        use core::fmt::Write;
        write_hundreds(f, hour as u8)?;
        f.write_char(':')?;
        write_hundreds(f, min as u8)?;
        f.write_char(':')?;
        write_hundreds(f, sec as u8)?;

        if nano == 0 {
            Ok(())
        } else if nano % 1_000_000 == 0 {
            write!(f, ".{:03}", nano / 1_000_000)
        } else if nano % 1_000 == 0 {
            write!(f, ".{:06}", nano / 1_000)
        } else {
            write!(f, ".{nano:09}")
        }
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(&self, f)
    }
}

pub(crate) fn write_hundreds(w: &mut (impl Write + ?Sized), n: u8) -> fmt::Result {
    if n >= 100 {
        return Err(fmt::Error);
    }

    let tens = b'0' + n / 10;
    let ones = b'0' + n % 10;
    w.write_char(tens as char)?;
    w.write_char(ones as char)
}
