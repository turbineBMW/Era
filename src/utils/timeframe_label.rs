use gettextrs::gettext;

use crate::system::ClockFormat;

#[derive(Debug)]
enum TzCase {
    BothSystem,
    AtLeastOneNotSystem,
}

#[derive(Debug)]
enum SameDayCase {
    Yesterday,
    Today,
    Tomorrow,
    NothingSpecial,
}

#[derive(Debug)]
enum DayCase {
    SameDay(SameDayCase),
    MultiDay,
}

#[derive(Debug)]
enum MonthCase {
    SameMonth,
    DifferentMonth,
}

#[derive(Debug)]
enum SameYearCase {
    CurrentYear,
    OtherYear,
}

#[derive(Debug)]
enum YearCase {
    SameYear(SameYearCase),
    DifferentYears,
}

/// Returns a human-readable label for an event timeframe.
pub fn all_day_timeframe_label(
    start: jiff::civil::Date,
    end: jiff::civil::Date,
    today: jiff::civil::Date,
) -> String {
    let day_case = if start == end {
        let yesterday = today.yesterday().unwrap();
        let tomorrow = today.tomorrow().unwrap();
        if start == yesterday {
            DayCase::SameDay(SameDayCase::Yesterday)
        } else if start == today {
            DayCase::SameDay(SameDayCase::Today)
        } else if start == tomorrow {
            DayCase::SameDay(SameDayCase::Tomorrow)
        } else {
            DayCase::SameDay(SameDayCase::NothingSpecial)
        }
    } else {
        DayCase::MultiDay
    };

    let month_case = if start.year() == end.year() && start.month() == end.month() {
        MonthCase::SameMonth
    } else {
        MonthCase::DifferentMonth
    };

    let year_case = if start.year() != end.year() {
        YearCase::DifferentYears
    } else if start.year() == today.year() {
        YearCase::SameYear(SameYearCase::CurrentYear)
    } else {
        YearCase::SameYear(SameYearCase::OtherYear)
    };

    match (&day_case, &month_case, &year_case) {
        (DayCase::SameDay(SameDayCase::Yesterday), _, _) => gettext("Yesterday"),
        (DayCase::SameDay(SameDayCase::Today), _, _) => gettext("Today"),
        (DayCase::SameDay(SameDayCase::Tomorrow), _, _) => gettext("Tomorrow"),

        (
            DayCase::SameDay(SameDayCase::NothingSpecial),
            _,
            YearCase::SameYear(SameYearCase::CurrentYear),
        ) => {
            // TRANSLATORS: date of a single all-day event, of the current year.
            // {month} is the month name (e.g. "June"), {day} is the day number (e.g. "12").
            gettext("{month} {day}")
                .replace("{month}", &start.strftime("%B").to_string())
                .replace("{day}", &start.strftime("%-d").to_string())
        }
        (DayCase::SameDay(SameDayCase::NothingSpecial), _, _) => {
            // TRANSLATORS: date of a single all-day event.
            // {month} is the month name (e.g. "June"), {day} is the day number (e.g. "12"),
            // {year} is the year (e.g. "2025").
            gettext("{month} {day}, {year}")
                .replace("{month}", &start.strftime("%B").to_string())
                .replace("{day}", &start.strftime("%-d").to_string())
                .replace("{year}", &start.strftime("%Y").to_string())
        }
        (
            DayCase::MultiDay,
            MonthCase::SameMonth,
            YearCase::SameYear(SameYearCase::CurrentYear),
        ) => {
            // e.g. "June 12 – 15"
            // TRANSLATORS: all-day event range within the same month of the current year.
            // {month} month name, {day_start} and {day_end} day numbers.
            gettext("{month} {day_start} \u{2013} {day_end}")
                .replace("{month}", &start.strftime("%B").to_string())
                .replace("{day_start}", &start.strftime("%-d").to_string())
                .replace("{day_end}", &end.strftime("%-d").to_string())
        }
        (DayCase::MultiDay, MonthCase::SameMonth, YearCase::SameYear(SameYearCase::OtherYear)) => {
            // e.g. "June 12 – 15, 2025"
            // TRANSLATORS: all-day event range within the same month of a past or future year.
            // {month} month name, {day_start} and {day_end} day numbers, {year} year.
            gettext("{month} {day_start} \u{2013} {day_end}, {year}")
                .replace("{month}", &start.strftime("%B").to_string())
                .replace("{day_start}", &start.strftime("%-d").to_string())
                .replace("{day_end}", &end.strftime("%-d").to_string())
                .replace("{year}", &start.strftime("%Y").to_string())
        }
        (
            DayCase::MultiDay,
            MonthCase::DifferentMonth,
            YearCase::SameYear(SameYearCase::CurrentYear),
        ) => {
            // e.g. "June 12 – July 3"
            // TRANSLATORS: all-day event range spanning different months in the current year.
            // {month_start} and {month_end} month names, {day_start} and {day_end} day numbers.
            gettext("{month_start} {day_start} \u{2013} {month_end} {day_end}")
                .replace("{month_start}", &start.strftime("%B").to_string())
                .replace("{day_start}", &start.strftime("%-d").to_string())
                .replace("{month_end}", &end.strftime("%B").to_string())
                .replace("{day_end}", &end.strftime("%-d").to_string())
        }
        (
            DayCase::MultiDay,
            MonthCase::DifferentMonth,
            YearCase::SameYear(SameYearCase::OtherYear),
        ) => {
            // e.g. "June 12 – July 3, 2025"
            // TRANSLATORS: all-day event range spanning different months in a past or future year.
            // {month_start} and {month_end} month names, {day_start} and {day_end} day numbers,
            // {year} the shared year.
            gettext("{month_start} {day_start} \u{2013} {month_end} {day_end}, {year}")
                .replace("{month_start}", &start.strftime("%B").to_string())
                .replace("{day_start}", &start.strftime("%-d").to_string())
                .replace("{month_end}", &end.strftime("%B").to_string())
                .replace("{day_end}", &end.strftime("%-d").to_string())
                .replace("{year}", &start.strftime("%Y").to_string())
        }
        (DayCase::MultiDay, _, YearCase::DifferentYears) => {
            // e.g. "June 12, 2024 – January 3, 2025"
            // TRANSLATORS: all-day event range spanning different years.
            // {month_start} and {month_end} month names, {day_start} and {day_end} day numbers,
            // {year_start} and {year_end} years.
            gettext("{month_start} {day_start}, {year_start} \u{2013} {month_end} {day_end}, {year_end}")
                    .replace("{month_start}", &start.strftime("%B").to_string())
                    .replace("{day_start}", &start.strftime("%-d").to_string())
                    .replace("{year_start}", &start.strftime("%Y").to_string())
                    .replace("{month_end}", &end.strftime("%B").to_string())
                    .replace("{day_end}", &end.strftime("%-d").to_string())
                    .replace("{year_end}", &end.strftime("%Y").to_string())
        }
    }
}

/// Returns a human-readable label for a timeslot event timeframe.
pub fn timeslot_timeframe_label(
    start: jiff::Zoned,
    end: jiff::Zoned,
    today: jiff::Zoned,
    clock_format: ClockFormat,
) -> String {
    let tz_start = start.time_zone().iana_name().unwrap();
    let tz_end = end.time_zone().iana_name().unwrap();

    let day_case = if start.date() == end.date() {
        let yesterday = today.yesterday().unwrap();
        let tomorrow = today.tomorrow().unwrap();
        if start.date() == yesterday.date() {
            DayCase::SameDay(SameDayCase::Yesterday)
        } else if start.date() == today.date() {
            DayCase::SameDay(SameDayCase::Today)
        } else if start.date() == tomorrow.date() {
            DayCase::SameDay(SameDayCase::Tomorrow)
        } else {
            DayCase::SameDay(SameDayCase::NothingSpecial)
        }
    } else {
        DayCase::MultiDay
    };

    let month_case = if start.year() == end.year() && start.month() == end.month() {
        MonthCase::SameMonth
    } else {
        MonthCase::DifferentMonth
    };

    let year_case = if start.year() != end.year() {
        YearCase::DifferentYears
    } else if start.year() == today.year() {
        YearCase::SameYear(SameYearCase::CurrentYear)
    } else {
        YearCase::SameYear(SameYearCase::OtherYear)
    };

    let tz_case = {
        let today = today.time_zone().iana_name().unwrap();
        match (tz_start == today, tz_end == today) {
            (true, true) => TzCase::BothSystem,
            _ => TzCase::AtLeastOneNotSystem,
        }
    };

    let time_fmt = match clock_format {
        ClockFormat::TwelveHours => "%-I:%M %p",
        ClockFormat::TwentyFourHours => "%H:%M",
    };
    let time_start = start.strftime(time_fmt).to_string();
    let time_end = end.strftime(time_fmt).to_string();

    match (&day_case, &month_case, &year_case, &tz_case) {
        (DayCase::SameDay(_), _, _, TzCase::BothSystem) => {
            // e.g. "June 12, 14:00 – 15:30"
            // TRANSLATORS: timed event on a single day, system timezone (not shown).
            // {month} month name, {day} day number, {time_start} and {time_end} clock times.
            gettext("{month} {day}, {time_start} \u{2013} {time_end}")
                .replace("{month}", &start.strftime("%B").to_string())
                .replace("{day}", &start.strftime("%-d").to_string())
                .replace("{time_start}", &time_start)
                .replace("{time_end}", &time_end)
        }
        (DayCase::SameDay(_), _, _, TzCase::AtLeastOneNotSystem) => {
            // e.g. "June 12, 14:00 (America/New_York) – 15:30 (Europe/Paris)"
            // TRANSLATORS: timed event on a single day, each time in a different non-system
            // timezone. {month}, {day}, {time_start}, {time_end} as above;
            // {tz_start} and {tz_end} the respective IANA timezones.
            gettext("{month} {day}, {time_start} ({tz_start}) \u{2013} {time_end} ({tz_end})")
                .replace("{month}", &start.strftime("%B").to_string())
                .replace("{day}", &start.strftime("%-d").to_string())
                .replace("{time_start}", &time_start)
                .replace("{time_end}", &time_end)
                .replace("{tz_start}", tz_start)
                .replace("{tz_end}", tz_end)
        }
        (
            DayCase::MultiDay,
            MonthCase::DifferentMonth | MonthCase::SameMonth,
            YearCase::SameYear(SameYearCase::CurrentYear),
            TzCase::BothSystem,
        ) => {
            // e.g. "June 12, 14:00 – July 3, 15:30"
            // TRANSLATORS: timed event spanning months within the current year, system timezone.
            // {month_start}, {day_start}, {time_start} start date/time;
            // {month_end}, {day_end}, {time_end} end date/time.
            gettext("{month_start} {day_start}, {time_start} \u{2013} {month_end} {day_end}, {time_end}")
                .replace("{month_start}", &start.strftime("%B").to_string())
                .replace("{day_start}", &start.strftime("%-d").to_string())
                .replace("{time_start}", &time_start)
                .replace("{month_end}", &end.strftime("%B").to_string())
                .replace("{day_end}", &end.strftime("%-d").to_string())
                .replace("{time_end}", &time_end)
        }
        (
            DayCase::MultiDay,
            MonthCase::DifferentMonth | MonthCase::SameMonth,
            YearCase::SameYear(SameYearCase::CurrentYear),
            TzCase::AtLeastOneNotSystem,
        ) => {
            // e.g. "June 12, 14:00 (America/New_York) – July 3, 15:30 (Europe/Paris)"
            // TRANSLATORS: cross-month current-year range; each time in a different non-system
            // timezone. {month_start}, {day_start}, {time_start}, {month_end}, {day_end},
            // {time_end} as above; {tz_start} and {tz_end} respective IANA timezones.
            gettext("{month_start} {day_start}, {time_start} ({tz_start}) \u{2013} {month_end} {day_end}, {time_end} ({tz_end})")
                .replace("{month_start}", &start.strftime("%B").to_string())
                .replace("{day_start}", &start.strftime("%-d").to_string())
                .replace("{time_start}", &time_start)
                .replace("{tz_start}", tz_start)
                .replace("{month_end}", &end.strftime("%B").to_string())
                .replace("{day_end}", &end.strftime("%-d").to_string())
                .replace("{time_end}", &time_end)
                .replace("{tz_end}", tz_end)
        }
        (
            DayCase::MultiDay,
            MonthCase::DifferentMonth | MonthCase::SameMonth,
            YearCase::SameYear(SameYearCase::OtherYear),
            TzCase::BothSystem,
        ) => {
            // e.g. "June 12, 14:00 – July 3, 15:30, 2025"
            // TRANSLATORS: timed event spanning months within a past or future year, system
            // timezone. {month_start}, {day_start}, {time_start}, {month_end}, {day_end},
            // {time_end} as above; {year} the shared year.
            gettext("{month_start} {day_start}, {time_start} \u{2013} {month_end} {day_end}, {time_end}, {year}")
                .replace("{month_start}", &start.strftime("%B").to_string())
                .replace("{day_start}", &start.strftime("%-d").to_string())
                .replace("{time_start}", &time_start)
                .replace("{month_end}", &end.strftime("%B").to_string())
                .replace("{day_end}", &end.strftime("%-d").to_string())
                .replace("{time_end}", &time_end)
                .replace("{year}", &start.strftime("%Y").to_string())
        }
        (
            DayCase::MultiDay,
            MonthCase::DifferentMonth | MonthCase::SameMonth,
            YearCase::SameYear(SameYearCase::OtherYear),
            TzCase::AtLeastOneNotSystem,
        ) => {
            // e.g. "June 12, 14:00 (America/New_York) – July 3, 15:30 (Europe/Paris), 2025"
            // TRANSLATORS: cross-month past/future-year range; each time in a different non-system
            // timezone. {month_start}, {day_start}, {time_start}, {month_end}, {day_end},
            // {time_end}, {year} as above; {tz_start} and {tz_end} respective IANA timezones.
            gettext("{month_start} {day_start}, {time_start} ({tz_start}) \u{2013} {month_end} {day_end}, {time_end} ({tz_end}), {year}")
                .replace("{month_start}", &start.strftime("%B").to_string())
                .replace("{day_start}", &start.strftime("%-d").to_string())
                .replace("{time_start}", &time_start)
                .replace("{tz_start}", tz_start)
                .replace("{month_end}", &end.strftime("%B").to_string())
                .replace("{day_end}", &end.strftime("%-d").to_string())
                .replace("{time_end}", &time_end)
                .replace("{year}", &start.strftime("%Y").to_string())
                .replace("{tz_end}", tz_end)
        }
        (DayCase::MultiDay, _, YearCase::DifferentYears, TzCase::BothSystem) => {
            // e.g. "June 12, 2024, 14:00 – January 3, 2025, 15:30"
            // TRANSLATORS: timed event spanning different years, system timezone.
            // {month_start}, {day_start}, {year_start}, {time_start} start date/time;
            // {month_end}, {day_end}, {year_end}, {time_end} end date/time.
            gettext("{month_start} {day_start}, {year_start}, {time_start} \u{2013} {month_end} {day_end}, {year_end}, {time_end}")
                .replace("{month_start}", &start.strftime("%B").to_string())
                .replace("{day_start}", &start.strftime("%-d").to_string())
                .replace("{year_start}", &start.strftime("%Y").to_string())
                .replace("{time_start}", &time_start)
                .replace("{month_end}", &end.strftime("%B").to_string())
                .replace("{day_end}", &end.strftime("%-d").to_string())
                .replace("{year_end}", &end.strftime("%Y").to_string())
                .replace("{time_end}", &time_end)
        }
        (DayCase::MultiDay, _, YearCase::DifferentYears, TzCase::AtLeastOneNotSystem) => {
            // e.g. "June 12, 2024, 14:00 (America/New_York) – January 3, 2025, 15:30
            // (Europe/Paris)" TRANSLATORS: different-year range; each time in a
            // different non-system timezone. {month_start}, {day_start}, {year_start},
            // {time_start}, {month_end}, {day_end}, {year_end}, {time_end} as above;
            // {tz_start} and {tz_end} respective IANA timezones.
            gettext("{month_start} {day_start}, {year_start}, {time_start} ({tz_start}) \u{2013} {month_end} {day_end}, {year_end}, {time_end} ({tz_end})")
                .replace("{month_start}", &start.strftime("%B").to_string())
                .replace("{day_start}", &start.strftime("%-d").to_string())
                .replace("{year_start}", &start.strftime("%Y").to_string())
                .replace("{time_start}", &time_start)
                .replace("{tz_start}", tz_start)
                .replace("{month_end}", &end.strftime("%B").to_string())
                .replace("{day_end}", &end.strftime("%-d").to_string())
                .replace("{year_end}", &end.strftime("%Y").to_string())
                .replace("{time_end}", &time_end)
                .replace("{tz_end}", tz_end)
        }
    }
}
