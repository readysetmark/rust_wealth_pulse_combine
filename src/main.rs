extern crate combine;

use combine::{char, digit, many, parser, Parser, ParserExt, ParseResult, ParseError};
use combine::combinator::FnParser;
use combine::primitives::{Consumed, State, Stream};

#[derive(PartialEq, Debug)]
struct Date {
	year: i32,
	month: i32,
	day: i32
}



/// Gets the current line number.
fn line_number<I>(input: State<I>) -> ParseResult<i32, I>
where I: Stream<Item=char> {
	Result::Ok((input.position.line, Consumed::Empty(input)))
}

#[test]
fn line_number_test() {
	let (line_num, remaining_input) = parser(line_number).parse("hello").unwrap();
	assert_eq!(line_num, 1);
	assert_eq!(remaining_input, "hello");
}



/// Takes a tuple of digit characters and converts them to an i32
fn two_digits_to_int((x, y): (char, char)) -> i32 {
    let x = x.to_digit(10).expect("digit");
    let y = y.to_digit(10).expect("digit");
    (x * 10 + y) as i32
}

#[test]
fn two_digits_to_int_test() {
	let two_digits = ('2', '7');
	let result = two_digits_to_int(two_digits);
	assert_eq!(result, 27);
}

/// Wrapped parser for parsing two digits. e.g. 17
fn two_digits<I>() -> FnParser<I, fn (State<I>) -> ParseResult<i32, I>>
where I: Stream<Item=char> {
    fn two_digits_<I>(input: State<I>) -> ParseResult<i32, I>
    where I: Stream<Item=char> {
    	println!("State.position = {:?}", input.position);

        (digit(), digit())
            .map(two_digits_to_int)
            .parse_state(input)
    }
    parser(two_digits_)
}

#[test]
fn two_digits_test() {
	let parse_result : Result<(i32, &str), ParseError<&str>> = two_digits().parse("09");
	let (result, _remaining_input) = parse_result.unwrap();
	assert_eq!(result, 9);
}



/// Parses a date. e.g. 2015-10-17
fn date<I>(input: State<I>) -> ParseResult<Date, I>
where I: Stream<Item=char> {
	(many::<String, _>(digit()), char('-'), two_digits(), char('-'), two_digits())
		.map(|(year, _, month, _, day)| {
			Date {
				year: year.parse().unwrap(),
				month: month,
				day: day
			}
		})
		.parse_state(input)
}

#[test]
fn date_test() {
	let parse_result : Result<(Date, &str), ParseError<&str>> = parser(date).parse("2015-10-17");
	let (result, _remaining_input) = parse_result.unwrap();
	assert_eq!(result.year, 2015);
	assert_eq!(result.month, 10);
	assert_eq!(result.day, 17);
}



fn main() {
	let result : Result<(Date, &str), ParseError<&str>> = parser(date).parse("2015-10-17");

	match result {
		Ok((date, remaining_input)) => {
			println!("{:?}", date);
			println!("{:?}", remaining_input)
		},
		Err(err) => println!("{}", err)
	}
}
