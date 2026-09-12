use jiff::ToSpan;

use crate::utils::WeekDay;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, glib::Boxed)]
#[boxed_type(name = "EraDate")]
pub struct Date(jiff::civil::Date);

impl From<jiff::civil::Date> for Date {
    fn from(value: jiff::civil::Date) -> Self {
        Self(value)
    }
}

impl From<glib::Date> for Date {
    fn from(value: glib::Date) -> Self {
        let year = value.year() as i16;
        let month = match value.month() {
            glib::DateMonth::January => 1,
            glib::DateMonth::February => 2,
            glib::DateMonth::March => 3,
            glib::DateMonth::April => 4,
            glib::DateMonth::May => 5,
            glib::DateMonth::June => 6,
            glib::DateMonth::July => 7,
            glib::DateMonth::August => 8,
            glib::DateMonth::September => 9,
            glib::DateMonth::October => 10,
            glib::DateMonth::November => 11,
            glib::DateMonth::December => 12,
            _ => unreachable!(),
        };
        let day = value.day() as i8;

        Self(jiff::civil::Date::new(year, month, day).unwrap())
    }
}

impl From<&glib::DateTime> for Date {
    fn from(value: &glib::DateTime) -> Self {
        let year = value.year() as i16;
        let month = value.month() as i8;
        let day = value.day_of_month() as i8;

        Self(jiff::civil::Date::new(year, month, day).unwrap())
    }
}

impl Date {
    pub fn to_jiff(self) -> jiff::civil::Date {
        self.0
    }

    pub fn to_glib_date(self) -> glib::Date {
        glib::Date::from_dmy(
            self.0.year() as u8,
            match self.0.month() {
                1 => glib::DateMonth::January,
                2 => glib::DateMonth::February,
                3 => glib::DateMonth::March,
                4 => glib::DateMonth::April,
                5 => glib::DateMonth::May,
                6 => glib::DateMonth::June,
                7 => glib::DateMonth::July,
                8 => glib::DateMonth::August,
                9 => glib::DateMonth::September,
                10 => glib::DateMonth::October,
                11 => glib::DateMonth::November,
                12 => glib::DateMonth::December,
                _ => unreachable!(),
            },
            self.0.day() as u16,
        )
        .unwrap()
    }

    pub fn to_glib_date_time_utc(self) -> glib::DateTime {
        glib::DateTime::from_utc(
            self.0.year() as i32,
            self.0.month() as i32,
            self.0.day() as i32,
            0,
            0,
            0.0,
        )
        .unwrap()
    }

    pub fn to_glib_date_time(self, timezone: &glib::TimeZone) -> glib::DateTime {
        glib::DateTime::new(
            timezone,
            self.0.year() as i32,
            self.0.month() as i32,
            self.0.day() as i32,
            0,
            0,
            0.0,
        )
        .unwrap()
    }

    pub fn previous_occurrence_of_weekday(&self, weekday: WeekDay) -> Self {
        let base = match weekday {
            WeekDay::Monday => 1,
            WeekDay::Tuesday => 2,
            WeekDay::Wednesday => 3,
            WeekDay::Thursday => 4,
            WeekDay::Friday => 5,
            WeekDay::Saturday => 6,
            WeekDay::Sunday => 7,
        };
        let offset = self.to_jiff().weekday().to_monday_one_offset();

        let go_back_by = (offset - base).rem_euclid(7);

        (self.to_jiff() - go_back_by.days()).into()
    }
}
