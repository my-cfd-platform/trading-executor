use chrono::{DateTime, Datelike, NaiveTime, Timelike, Utc, Weekday};
use my_nosql_contracts::{TradingInstrumentDayOff, TradingInstrumentNoSqlEntity};

use crate::TradingExecutorError;

pub fn validate_instrument_day_off(
    instrument: &TradingInstrumentNoSqlEntity,
) -> Result<(), TradingExecutorError> {
    for day_off in &instrument.days_off {
        validate_day_off(&day_off, Utc::now())?;
    }

    return Ok(());
}

pub fn validate_day_off(
    instrument: &TradingInstrumentDayOff,
    current_date: DateTime<Utc>,
) -> Result<(), TradingExecutorError> {
    let current_weekday = current_date.weekday();
    let current_time = current_date.time();

    let from_as_int = as_int(
        convert_csharp_int_day_to_rust_weekday(instrument.dow_from),
        instrument.time_from.parse().unwrap(),
    );

    let to_as_int = as_int(
        convert_csharp_int_day_to_rust_weekday(instrument.dow_to),
        instrument.time_to.parse().unwrap(),
    );

    let current_as_int = as_int(current_weekday, current_time);

    let is_first_case = from_as_int < to_as_int;

    let is_day_off = match is_first_case {
        true => from_as_int <= current_as_int && current_as_int <= to_as_int,
        false => {
            let first_case = current_as_int >= from_as_int;
            let second_case = current_as_int <= to_as_int;
            first_case || second_case
        }
    };

    if is_day_off {
        return Err(TradingExecutorError::DayOff);
    }

    return Ok(());
}

fn as_int(weekday: Weekday, time: NaiveTime) -> u32 {
    return weekday as u32 * 86400 + time.hour() * 3600 + time.minute() * 60 + time.second();
}

fn convert_csharp_int_day_to_rust_weekday(src: i32) -> Weekday {
    if src == 0 {
        return Weekday::Sun;
    }
    let src = src - 1;

    return Weekday::try_from(src as u8).unwrap();
}

#[cfg(test)]
mod test {
    use chrono::{TimeZone, Utc};
    use my_nosql_contracts::TradingInstrumentDayOff;

    use crate::validate_day_off;

    #[test]
    fn check_day_off_f_s_t_day_off() {
        let day_off = TradingInstrumentDayOff {
            dow_from: 1,
            time_from: "21:00:00".to_string(),
            dow_to: 2,
            time_to: "14:30:00".to_string(),
        };

        let date = Utc.ymd(2023, 11, 20).and_hms(22, 0, 0);

        let validate_result = validate_day_off(&day_off, date);

        assert_eq!(true, validate_result.is_err());
    }

    #[test]
    fn check_day_off_f_s_t_day_on() {
        let day_off = TradingInstrumentDayOff {
            dow_from: 1,
            time_from: "21:00:00".to_string(),
            dow_to: 2,
            time_to: "14:30:00".to_string(),
        };

        let date = Utc.ymd(2023, 11, 20).and_hms(20, 0, 0);

        let validate_result = validate_day_off(&day_off, date);

        assert_eq!(false, validate_result.is_err());
    }

    #[test]
    fn check_day_off_f_b_t_day_off() {
        let day_off = TradingInstrumentDayOff {
            dow_from: 5,
            time_from: "21:00:00".to_string(),
            dow_to: 2,
            time_to: "14:30:00".to_string(),
        };

        let date = Utc.ymd(2023, 11, 20).and_hms(22, 0, 0);

        let validate_result = validate_day_off(&day_off, date);

        assert_eq!(true, validate_result.is_err());
    }

    #[test]
    fn check_day_off_f_b_t_day_off_2() {
        let day_off = TradingInstrumentDayOff {
            dow_from: 5,
            time_from: "21:00:00".to_string(),
            dow_to: 2,
            time_to: "14:30:00".to_string(),
        };

        let date = Utc.ymd(2023, 11, 22).and_hms(14, 29, 0);

        let validate_result = validate_day_off(&day_off, date);

        assert_eq!(false, validate_result.is_err());
    }

    #[test]
    fn check_day_off_f_b_t_day_on() {
        let day_off = TradingInstrumentDayOff {
            dow_from: 5,
            time_from: "21:00:00".to_string(),
            dow_to: 2,
            time_to: "14:30:00".to_string(),
        };

        let date = Utc.ymd(2023, 11, 22).and_hms(14, 31, 0);

        let validate_result = validate_day_off(&day_off, date);

        assert_eq!(false, validate_result.is_err());
    }
}
