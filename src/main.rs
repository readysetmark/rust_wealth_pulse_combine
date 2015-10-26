extern crate combine;

use combine::{alpha_num, char, crlf, digit, many, many1, newline, optional,
	parser, satisfy, sep_by, sep_by1, Parser, ParserExt, ParseResult,
	ParseError};
use combine::combinator::FnParser;
use combine::primitives::{Consumed, State, Stream};


#[derive(PartialEq, Debug)]
enum AmountFormat {
	SymbolLeftNoSpace,
	SymbolLeftWithSpace,
	SymbolRightNoSpace,
	SymbolRightWithSpace
}

#[derive(PartialEq, Debug)]
enum TransactionStatus {
	Cleared,
	Uncleared
}

#[derive(PartialEq, Debug)]
struct Date {
	year: i32,
	month: i32,
	day: i32
}

#[derive(PartialEq, Debug)]
struct Header {
	line_number: i32,
	date: Date,
	status: TransactionStatus,
	code: Option<String>,
	payee: String,
	comment: Option<String>
}

#[derive(PartialEq, Debug)]
struct Symbol {
	value: String,
	quoted: bool
}

#[derive(PartialEq, Debug)]
struct Amount {
	value: String,
	symbol: Symbol,
	format: AmountFormat
}

#[derive(PartialEq, Debug)]
struct Price {
	date: Date,
	symbol: Symbol,
	amount: Amount
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



/// Parses at least one whitespace character (space or tab).
fn whitespace<I>(input: State<I>) -> ParseResult<String, I>
where I: Stream<Item=char> {
	many1::<String, _>(satisfy(|c| c == ' ' || c == '\t'))
		.parse_state(input)
}

#[test]
fn empty_whitespace_is_error()
{
	let result = parser(whitespace)
		.parse("")
		.map(|x| x.0);
	assert!(result.is_err());
}

#[test]
fn whitespace_space()
{
	let result = parser(whitespace)
		.parse(" ")
		.map(|x| x.0);
	assert_eq!(result, Ok(" ".to_string()));
}

#[test]
fn whitespace_tab()
{
	let result = parser(whitespace)
		.parse("\t")
		.map(|x| x.0);
	assert_eq!(result, Ok("\t".to_string()));
}



/// Parses a Unix or Windows style line endings
fn line_ending<I>(input: State<I>) -> ParseResult<String, I>
where I: Stream<Item=char> {
	crlf()
		.map(|x: char| x.to_string())
		.or(newline()
			.map(|x: char| x.to_string()))
		.parse_state(input)
}

#[test]
fn line_ending_unix() {
	let result = parser(line_ending)
		.parse("\n")
		.map(|x| x.0);
	assert_eq!(result, Ok("\n".to_string()));
}

#[test]
fn line_ending_windows() {
	let result = parser(line_ending)
		.parse("\r\n")
		.map(|x| x.0);
	assert_eq!(result, Ok("\n".to_string()));
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
fn status<I>(input: State<I>) -> ParseResult<TransactionStatus, I>
where I: Stream<Item=char> {
	char('*')
		.map(|_| TransactionStatus::Cleared)
		.or(char('!').map(|_| TransactionStatus::Uncleared))
		.parse_state(input)
}

#[test]
fn status_cleared() {
	let result = parser(status)
		.parse("*")
		.map(|x| x.0);
	assert_eq!(result, Ok(TransactionStatus::Cleared));
}

#[test]
fn status_uncleared() {
	let result = parser(status)
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



/// Parses a transaction header
fn header<I>(input: State<I>) -> ParseResult<Header,I>
where I: Stream<Item=char> {
	(
		parser(line_number),
		parser(date).skip(parser(whitespace)),
		parser(status).skip(parser(whitespace)),
		optional(parser(code).skip(parser(whitespace))),
		parser(payee),
		optional(parser(comment))
	)
		.map(|(line_num, date, status, code, payee, comment)| {
			Header {
				line_number: line_num,
				date: date,
				status: status,
				code: code,
				payee: payee,
				comment: comment
			}
		})
		.parse_state(input)
}

#[test]
fn full_header() {
	let result = parser(header)
		.parse("2015-10-20 * (conf# abc-123) Payee ;Comment")
		.map(|x| x.0);
	assert_eq!(result, Ok(Header {
		line_number: 1,
		date: Date {
			year: 2015,
			month: 10,
			day: 20
		},
		status: TransactionStatus::Cleared,
		code: Some("conf# abc-123".to_string()),
		payee: "Payee ".to_string(),
		comment: Some("Comment".to_string())
	}));
}

#[test]
fn header_with_code_and_no_comment() {
	let result = parser(header)
		.parse("2015-10-20 ! (conf# abc-123) Payee")
		.map(|x| x.0);
	assert_eq!(result, Ok(Header {
		line_number: 1,
		date: Date {
			year: 2015,
			month: 10,
			day: 20
		},
		status: TransactionStatus::Uncleared,
		code: Some("conf# abc-123".to_string()),
		payee: "Payee".to_string(),
		comment: None
	}));
}

#[test]
fn header_with_comment_and_no_code() {
	let result = parser(header)
		.parse("2015-10-20 * Payee ;Comment")
		.map(|x| x.0);
	assert_eq!(result, Ok(Header {
		line_number: 1,
		date: Date {
			year: 2015,
			month: 10,
			day: 20
		},
		status: TransactionStatus::Cleared,
		code: None,
		payee: "Payee ".to_string(),
		comment: Some("Comment".to_string())
	}));
}

#[test]
fn header_with_no_code_or_comment() {
	let result = parser(header)
		.parse("2015-10-20 * Payee")
		.map(|x| x.0);
	assert_eq!(result, Ok(Header {
		line_number: 1,
		date: Date {
			year: 2015,
			month: 10,
			day: 20
		},
		status: TransactionStatus::Cleared,
		code: None,
		payee: "Payee".to_string(),
		comment: None
	}));
}



/// Parses a sub-account name, which must be alphanumeric.
fn sub_account<I>(input: State<I>) -> ParseResult<String,I>
where I: Stream<Item=char> {
	many1(alpha_num())
		.parse_state(input)
}

#[test]
fn sub_account_alphanumeric() {
	let result = parser(sub_account)
		.parse("AZaz09")
		.map(|x| x.0);
	assert_eq!(result, Ok("AZaz09".to_string()));
}

#[test]
fn sub_account_can_start_with_digits() {
	let result = parser(sub_account)
		.parse("123abcABC")
		.map(|x| x.0);
	assert_eq!(result, Ok("123abcABC".to_string()));
}



/// Parses an account, made up of sub-accounts separated by colons.
fn account<I>(input: State<I>) -> ParseResult<Vec<String>,I>
where I: Stream<Item=char> {
	sep_by1(parser(sub_account), char(':'))
		.parse_state(input)
}

#[test]
fn account_multiple_level() {
	let result = parser(account)
		.parse("Expenses:Food:Groceries")
		.map(|x| x.0);
	assert_eq!(result, Ok(vec![
		"Expenses".to_string(),
		"Food".to_string(),
		"Groceries".to_string()
	]));
}

#[test]
fn account_single_level() {
	let result = parser(account)
		.parse("Expenses")
		.map(|x| x.0);
	assert_eq!(result, Ok(vec!["Expenses".to_string()]));
}



/// Parses a numeric quantity
fn quantity<I>(input: State<I>) -> ParseResult<String,I>
where I: Stream<Item=char> {
	(
		optional(char('-'))
			.map(|x| {
				match x {
					Some(_) => "-".to_string(),
					None => "".to_string()
				}
			}),
		satisfy(|c : char| c.is_digit(10)),
		many::<String, _>(satisfy(|c : char| {
			c.is_digit(10) || c == ',' || c == '.'
		}))
	)
		.map(|(neg_sign, first_digit, digits_or_separators)| {
			// TODO: need to return a numeric type here
			let qty = format!("{}{}{}",
				neg_sign,
				first_digit,
				digits_or_separators);
			qty.replace(",", "")
		})
		.parse_state(input)
}

#[test]
fn quantity_negative_no_fractional_part()
{
	let result = parser(quantity)
		.parse("-1110")
		.map(|x| x.0);
	assert_eq!(result, Ok("-1110".to_string()));
}

#[test]
fn quantity_positive_no_fractional_part()
{
	let result = parser(quantity)
		.parse("2,314")
		.map(|x| x.0);
	assert_eq!(result, Ok("2314".to_string()));
}

#[test]
fn quantity_negative_with_fractional_part()
{
	let result = parser(quantity)
		.parse("-1,110.38")
		.map(|x| x.0);
	assert_eq!(result, Ok("-1110.38".to_string()));
}

#[test]
fn quantity_positive_with_fractional_part()
{
	let result = parser(quantity)
		.parse("24521.793")
		.map(|x| x.0);
	assert_eq!(result, Ok("24521.793".to_string()));
}



/// Parses a quoted symbol
fn quoted_symbol<I>(input: State<I>) -> ParseResult<Symbol, I>
where I: Stream<Item=char> {
	(char('\"'), many1(satisfy(|c| c != '\"' && c != '\r' && c != '\n')), char('\"'))
		.map(|(_, symbol, _)| Symbol {
			value: symbol,
			quoted: true
		})
		.parse_state(input)
}

#[test]
fn quoted_symbol_test() {
	let result = parser(quoted_symbol)
		.parse("\"MUTF2351\"")
		.map(|x| x.0);
	assert_eq!(result, Ok(Symbol {
		value: "MUTF2351".to_string(),
		quoted: true
	}));
}



/// Parses an unquoted symbol
fn unquoted_symbol<I>(input: State<I>) -> ParseResult<Symbol, I>
where I: Stream<Item=char> {
	many1(satisfy(|c| "-0123456789; \"\t\r\n".chars().all(|s| s != c)))
		.map(|symbol| Symbol {
			value: symbol,
			quoted: false
		})
		.parse_state(input)
}

#[test]
fn unquoted_symbol_just_symbol() {
	let result = parser(unquoted_symbol)
		.parse("$")
		.map(|x| x.0);
	assert_eq!(result, Ok(Symbol {
		value: "$".to_string(),
		quoted: false
	}));
}

#[test]
fn unquoted_symbol_symbol_and_letters() {
	let result = parser(unquoted_symbol)
		.parse("US$")
		.map(|x| x.0);
	assert_eq!(result, Ok(Symbol {
		value: "US$".to_string(),
		quoted: false
	}));
}

#[test]
fn unquoted_symbol_just_letters() {
	let result = parser(unquoted_symbol)
		.parse("AAPL")
		.map(|x| x.0);
	assert_eq!(result, Ok(Symbol {
		value: "AAPL".to_string(),
		quoted: false
	}));
}



/// Parses a quoted or unquoted symbol
fn symbol<I>(input: State<I>) -> ParseResult<Symbol, I>
where I: Stream<Item=char> {
	parser(quoted_symbol)
		.or(parser(unquoted_symbol))
		.parse_state(input)
}

#[test]
fn symbol_unquoted_test() {
	let result = parser(symbol)
		.parse("$")
		.map(|x| x.0);
	assert_eq!(result, Ok(Symbol {
		value: "$".to_string(),
		quoted: false
	}));
}

#[test]
fn symbol_quoted_test() {
	let result = parser(symbol)
		.parse("\"MUTF2351\"")
		.map(|x| x.0);
	assert_eq!(result, Ok(Symbol {
		value: "MUTF2351".to_string(),
		quoted: true
	}));
}



/// Parses an amount in the format of symbol then quantity.
fn amount_symbol_then_quantity<I>(input: State<I>) -> ParseResult<Amount, I>
where I: Stream<Item=char> {
	(parser(symbol), optional(parser(whitespace)), parser(quantity))
		.map(|(symbol, opt_whitespace, quantity)| {
			let format = match opt_whitespace {
				Some(_) => AmountFormat::SymbolLeftWithSpace,
				None => AmountFormat::SymbolLeftNoSpace
			};
			Amount {
				value: quantity,
				symbol: symbol,
				format: format
			}
		})
		.parse_state(input)
}

#[test]
fn amount_symbol_then_quantity_no_whitespace() {
	let result = parser(amount_symbol_then_quantity)
		.parse("$13,245.00")
		.map(|x| x.0);
	assert_eq!(result, Ok(Amount {
		value: "13245.00".to_string(),
		symbol: Symbol {
			value: "$".to_string(),
			quoted: false
		},
		format: AmountFormat::SymbolLeftNoSpace
	}));
}

#[test]
fn amount_symbol_then_quantity_with_whitespace() {
	let result = parser(amount_symbol_then_quantity)
		.parse("$ 13,245.00")
		.map(|x| x.0);
	assert_eq!(result, Ok(Amount {
		value: "13245.00".to_string(),
		symbol: Symbol {
			value: "$".to_string(),
			quoted: false
		},
		format: AmountFormat::SymbolLeftWithSpace
	}));
}



/// Parses an amount in the format of quantity then symbol.
fn amount_quantity_then_symbol<I>(input: State<I>) -> ParseResult<Amount, I>
where I: Stream<Item=char> {
	(parser(quantity), optional(parser(whitespace)), parser(symbol))
		.map(|(quantity, opt_whitespace, symbol)| {
			let format = match opt_whitespace {
				Some(_) => AmountFormat::SymbolRightWithSpace,
				None => AmountFormat::SymbolRightNoSpace
			};
			Amount {
				value: quantity,
				symbol: symbol,
				format: format
			}
		})
		.parse_state(input)
}

#[test]
fn amount_quantity_then_symbol_no_whitespace() {
	let result = parser(amount_quantity_then_symbol)
		.parse("13,245.463AAPL")
		.map(|x| x.0);
	assert_eq!(result, Ok(Amount {
		value: "13245.463".to_string(),
		symbol: Symbol {
			value: "AAPL".to_string(),
			quoted: false
		},
		format: AmountFormat::SymbolRightNoSpace
	}));
}

#[test]
fn amount_quantity_then_symbol_with_whitespace() {
	let result = parser(amount_quantity_then_symbol)
		.parse("13,245.463 \"MUTF2351\"")
		.map(|x| x.0);
	assert_eq!(result, Ok(Amount {
		value: "13245.463".to_string(),
		symbol: Symbol {
			value: "MUTF2351".to_string(),
			quoted: true
		},
		format: AmountFormat::SymbolRightWithSpace
	}));
}



/// Parses an amount or an inferred amount
fn amount<I>(input: State<I>) -> ParseResult<Amount, I>
where I: Stream<Item=char> {
	parser(amount_symbol_then_quantity)
		.or(parser(amount_quantity_then_symbol))
		.parse_state(input)
}

#[test]
fn amount_test_symbol_then_quantity() {
	let result = parser(amount)
		.parse("$13,245.46")
		.map(|x| x.0);
	assert_eq!(result, Ok(Amount {
		value: "13245.46".to_string(),
		symbol: Symbol {
			value: "$".to_string(),
			quoted: false
		},
		format: AmountFormat::SymbolLeftNoSpace
	}));
}

#[test]
fn amount_test_quantity_then_symbol() {
	let result = parser(amount)
		.parse("13,245.463 \"MUTF2351\"")
		.map(|x| x.0);
	assert_eq!(result, Ok(Amount {
		value: "13245.463".to_string(),
		symbol: Symbol {
			value: "MUTF2351".to_string(),
			quoted: true
		},
		format: AmountFormat::SymbolRightWithSpace
	}));
}





/// Parses a price entry
fn price<I>(input: State<I>) -> ParseResult<Price, I>
where I: Stream<Item=char> {
	(
		char('P').skip(parser(whitespace)),
		parser(date).skip(parser(whitespace)),
		parser(symbol).skip(parser(whitespace)),
		parser(amount)
	)
		.map(|(_, date, symbol, amount)| Price {
			date: date,
			symbol: symbol,
			amount: amount
		})
		.parse_state(input)
}

#[test]
fn price_test() {
	let result = parser(price)
		.parse("P 2015-10-25 \"MUTF2351\" $5.42")
		.map(|x| x.0);
	assert_eq!(result, Ok(Price {
		date: Date {
			year: 2015,
			month: 10,
			day: 25
		},
		symbol: Symbol {
			value: "MUTF2351".to_string(),
			quoted: true
		},
		amount: Amount {
			value: "5.42".to_string(),
			symbol: Symbol {
				value: "$".to_string(),
				quoted: false
			},
			format: AmountFormat::SymbolLeftNoSpace
		}
	}));
}



/// Parses a price DB file, which contains only price entries.
fn price_db<I>(input: State<I>) -> ParseResult<Vec<Price>, I>
where I: Stream<Item=char> {
	sep_by(parser(price), parser(line_ending))
		.parse_state(input)
}

#[test]
fn price_db_no_records() {
	let result = parser(price_db)
		.parse("")
		.map(|x| x.0);
	assert_eq!(result, Ok(vec![]));
}

#[test]
fn price_db_one_record() {
	let result = parser(price_db)
		.parse("P 2015-10-25 \"MUTF2351\" $5.42")
		.map(|x| x.0);
	assert_eq!(result, Ok(vec![
		Price {
			date: Date {
				year: 2015,
				month: 10,
				day: 25
			},
			symbol: Symbol {
				value: "MUTF2351".to_string(),
				quoted: true
			},
			amount: Amount {
				value: "5.42".to_string(),
				symbol: Symbol {
					value: "$".to_string(),
					quoted: false
				},
				format: AmountFormat::SymbolLeftNoSpace
			}
		}
	]));
}

#[test]
fn price_db_multiple_records() {
	let result = parser(price_db)
		.parse("\
			P 2015-10-23 \"MUTF2351\" $5.42\n\
			P 2015-10-25 \"MUTF2351\" $5.98\n\
			P 2015-10-25 AAPL $313.38\
		")
		.map(|x| x.0);
	assert_eq!(result, Ok(vec![
		Price {
			date: Date {
				year: 2015,
				month: 10,
				day: 23
			},
			symbol: Symbol {
				value: "MUTF2351".to_string(),
				quoted: true
			},
			amount: Amount {
				value: "5.42".to_string(),
				symbol: Symbol {
					value: "$".to_string(),
					quoted: false
				},
				format: AmountFormat::SymbolLeftNoSpace
			}
		},
		Price {
			date: Date {
				year: 2015,
				month: 10,
				day: 25
			},
			symbol: Symbol {
				value: "MUTF2351".to_string(),
				quoted: true
			},
			amount: Amount {
				value: "5.98".to_string(),
				symbol: Symbol {
					value: "$".to_string(),
					quoted: false
				},
				format: AmountFormat::SymbolLeftNoSpace
			}
		},
		Price {
			date: Date {
				year: 2015,
				month: 10,
				day: 25
			},
			symbol: Symbol {
				value: "AAPL".to_string(),
				quoted: false
			},
			amount: Amount {
				value: "313.38".to_string(),
				symbol: Symbol {
					value: "$".to_string(),
					quoted: false
				},
				format: AmountFormat::SymbolLeftNoSpace
			}
		}
	]));
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
