fn main() {
 let body = "abc";
 let start = 0usize; let end = 99usize;
 let absent_or_non_string_quote: Option<&str> = None;
 assert_eq!(body.get(start..end), absent_or_non_string_quote, "quote differs from retained full source");
 println!("guard_accepted_invalid_span_and_absent_quote");
}
