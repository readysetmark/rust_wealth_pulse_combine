extern crate combine;

use combine::{char, digit, many, many1, parser, satisfy, Parser, ParserExt,
	ParseResult, ParseError};
use combine::combinator::FnParser;
use combine::primitives::{Consumed, State, Stream};

#[derive(PartialEq, Debug)]
struct Date {
	year: i32,
	month: i32,
	day: i32
}

#[derive(PartialEq, Debug)]
enum TransactionStatus {
	Cleared,
	Uncleared
}


/// Gets the current line number.
fn line_number<I>(input: State<I>) -> ParseResult<i32, I>
where I: Stream<Item=char> {
	Ok((input.position.line, Consumed::Empty(input)))
}

#[test]
fn line_number_test() {
	let (line_num, remaining_input) = parser(line_number)
		.parse("hello")
		.unwrap();
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
	let result = two_digits_to_int(('2', '7'));
	assert_eq!(result, 27);
}



/// Wrapped parser for parsing two digits. e.g. 17
fn two_digits<I>() -> FnParser<I, fn (State<I>) -> ParseResult<i32, I>>
where I: Stream<Item=char> {
    fn two_digits_<I>(input: State<I>) -> ParseResult<i32, I>
    where I: Stream<Item=char> {
        (digit(), digit())
            .map(two_digits_to_int)
            .parse_state(input)
    }
    parser(two_digits_)
}

#[test]
fn two_digits_test() {
	let result = two_digits()
		.parse("09")
		.map(|x| x.0);
	assert_eq!(result, Ok(9));
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
	let result = parser(date)
		.parse("2015-10-17")
		.map(|x| x.0);
	assert_eq!(result, Ok(Date {
		year: 2015,
		month: 10,
		day: 17
	}));
}



/// Parses transaction status token. e.g. * (cleared) or ! (uncleared)
fn transaction_status<I>(input: State<I>) -> ParseResult<TransactionStatus, I>
where I: Stream<Item=char> {
	char('*')
		.map(|_| TransactionStatus::Cleared)
		.or(char('!').map(|_| TransactionStatus::Uncleared))
		.parse_state(input)
}

#[test]
fn transaction_status_cleared() {
	let result = parser(transaction_status)
		.parse("*")
		.map(|x| x.0);
	assert_eq!(result, Ok(TransactionStatus::Cleared));
}

#[test]
fn transaction_status_uncleared() {
	let result = parser(transaction_status)
		.parse("!")
		.map(|x| x.0);
	assert_eq!(result, Ok(TransactionStatus::Uncleared));
}



/// Parses transaction code. e.g. (cheque #802)
fn code<I>(input: State<I>) -> ParseResult<String, I>
where I: Stream<Item=char> {
	(char('('), many(satisfy(|c| c != '\r' && c != '\n' && c != ')')), char(')'))
		.map(|(_, code, _)| code)
		.parse_state(input)
}

#[test]
fn empty_code() {
	let result = parser(code)
		.parse("()")
		.map(|x| x.0);
	assert!(result.unwrap().is_empty());
}

#[test]
fn short_code() {
	let result = parser(code)
		.parse("(89)")
		.map(|x| x.0);
	assert_eq!(result, Ok("89".to_string()));
}

#[test]
fn long_code() {
	let result = parser(code)
		.parse("(conf# abc-123-DEF)")
		.map(|x| x.0);
	assert_eq!(result, Ok("conf# abc-123-DEF".to_string()));
}



/// Parses a payee.
fn payee<I>(input: State<I>) -> ParseResult<String,I>
where I: Stream<Item=char> {
	many1(satisfy(|c| c != ';' && c != '\n' && c != '\r'))
		.parse_state(input)
}

#[test]
fn empty_payee_is_error() {
	let result = parser(payee)
		.parse("")
		.map(|x| x.0);
	assert!(result.is_err());
}

#[test]
fn single_character_payee() {
	let result = parser(payee)
		.parse("Z")
		.map(|x| x.0);
	assert_eq!(result, Ok("Z".to_string()));
}

#[test]
fn short_payee() {
	let result = parser(payee)
		.parse("WonderMart")
		.map(|x| x.0);
	assert_eq!(result, Ok("WonderMart".to_string()));
}

#[test]
fn long_payee() {
	let result = parser(payee)
		.parse("WonderMart - groceries, kitchen supplies (pot), light bulbs")
		.map(|x| x.0);
	assert_eq!(result,
		Ok("WonderMart - groceries, kitchen supplies (pot), light bulbs".to_string()));
}



/// Parses a comment.
fn comment<I>(input: State<I>) -> ParseResult<String,I>
where I: Stream<Item=char> {
	(char(';'), many(satisfy(|c| c != '\r' && c != '\n')))
		.map(|(_, payee)| payee)
		.parse_state(input)
}

#[test]
fn empty_comment() {
	let result = parser(comment)
		.parse(";")
		.map(|x| x.0);
	assert!(result.unwrap().is_empty());
}

#[test]
fn comment_no_leading_space() {
	let result = parser(comment)
		.parse(";Comment")
		.map(|x| x.0);
	assert_eq!(result, Ok("Comment".to_string()));
}

#[test]
fn comment_with_leading_space() {
	let result = parser(comment)
		.parse("; Comment")
		.map(|x| x.0);
	assert_eq!(result, Ok(" Comment".to_string()));
}



fn main() {
	let result : Result<(String, &str), ParseError<&str>> = parser(payee).parse("");

	println!("{:?}", result);

	match result {
		Ok((date, remaining_input)) => {
			println!("{:?}", date);
			println!("{:?}", remaining_input)
		},
		Err(err) => println!("{}", err)
	}
}
