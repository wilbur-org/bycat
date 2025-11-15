use core::str::FromStr;

use udled::{
    AsChar, Buffer, EOF, Input, Reader, Tokenizer, TokenizerExt,
    tokenizers::{Digit, Peek, or},
};
use udled_tokenizers::Integer;

use crate::{Date, DateTime, Time, time::TimeZone};

const TWO_DIGITS: (Digit, Digit) = (Digit(10), Digit(10));
const FOUR_DIGITS: (Digit, Digit, Digit, Digit) = (Digit(10), Digit(10), Digit(10), Digit(10));

pub struct DateParser;

impl<'input, B> Tokenizer<'input, B> for DateParser
where
    B: Buffer<'input>,
    B::Item: AsChar,
{
    type Token = Date;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, udled::Error> {
        let (year, _, month, _, day) = reader.parse((
            FOUR_DIGITS.into_integer(10),
            '-',
            TWO_DIGITS.into_integer(10),
            '-',
            TWO_DIGITS.into_integer(10),
        ))?;

        Ok(Date::new(year.value as _, month.value as _, day.value as _))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), udled::Error> {
        reader.eat((FOUR_DIGITS, '-', TWO_DIGITS, '-', TWO_DIGITS))
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(Peek(FOUR_DIGITS))
    }
}

pub struct TimeParser;

impl<'input, B> Tokenizer<'input, B> for TimeParser
where
    B: Buffer<'input>,
    B::Item: AsChar,
{
    type Token = Time;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, udled::Error> {
        let parser = TWO_DIGITS.into_integer(10);

        let (hour, _, min, _, secs) = reader.parse((&parser, ':', &parser, ':', &parser))?;

        let nano = if reader.is('.') {
            reader.eat('.')?;
            let nano = reader.parse(Integer)?;
            nano.value as u32
        } else {
            0
        };

        Ok(Time::from_hmsn(
            hour.value as _,
            min.value as _,
            secs.value as _,
            nano,
        ))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), udled::Error> {
        reader.eat((TWO_DIGITS, ':', TWO_DIGITS, ':', TWO_DIGITS))?;

        if reader.is('.') {
            reader.eat('.')?;
            reader.eat(Integer)?;
        }

        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(Peek((TWO_DIGITS, ':')))
    }
}

pub struct TimeZoneParser;

impl<'input, B> Tokenizer<'input, B> for TimeZoneParser
where
    B: Buffer<'input>,
    B::Item: AsChar,
{
    type Token = TimeZone;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, udled::Error> {
        let (sign, h, m) = or(
            // Utc
            'z'.or('Z').map_ok(|_| (0i32, 0u32, 0u32)),
            // Offset
            (
                // Sign
                '-'.map_ok(|_| -1).or('+'.map_ok(|_| 1)),
                // Hour
                TWO_DIGITS.into_integer(10).map_ok(|m| m.value as u32),
                ':'.optional(),
                // Optional Minute
                TWO_DIGITS
                    .into_integer(10)
                    .optional()
                    .map_ok(|m| m.map(|m| m.value as u32).unwrap_or(0)),
            )
                .map_ok(|(sign, h, _, m)| (sign.unify(), h, m)),
        )
        .map_ok(|m| m.unify())
        .parse(reader)?;

        let secs = h * 3600 + m * 60;

        Ok(TimeZone::from_secs(secs as i32 * sign))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), udled::Error> {
        reader.eat(or(
            // UTC
            'z'.or('Z'),
            // Offset
            (
                '-'.or('+'),
                TWO_DIGITS,
                ':'.optional(),
                TWO_DIGITS.optional(),
            ),
        ))
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(Peek(('+'.or('-'), TWO_DIGITS)))
    }
}

pub struct DateTimeParser;

impl<'input, B> Tokenizer<'input, B> for DateTimeParser
where
    B: Buffer<'input>,
    B::Item: AsChar,
{
    type Token = DateTime;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, udled::Error> {
        let (date, _, time, time_zone) =
            reader.parse((DateParser, 'T'.or(' '), TimeParser, TimeZoneParser))?;

        Ok(DateTime::new(date, time, time_zone))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), udled::Error> {
        reader.eat((DateParser, 'T'.or(' '), TimeParser, TimeZoneParser))?;
        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(Peek((FOUR_DIGITS, '-')))
    }
}

impl FromStr for Time {
    type Err = udled::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Input::new(s.as_bytes()).parse((TimeParser, EOF).map_ok(|m| m.0))
    }
}

impl FromStr for Date {
    type Err = udled::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Input::new(s.as_bytes()).parse((DateParser, EOF).map_ok(|m| m.0))
    }
}

impl FromStr for DateTime {
    type Err = udled::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Input::new(s.as_bytes()).parse((DateTimeParser, EOF).map_ok(|m| m.0))
    }
}
