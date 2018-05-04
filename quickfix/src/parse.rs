#![allow(non_snake_case)]

use nom::{eol, space, alpha1, alphanumeric1, be_f64};

//--- FAIL: TestStableDeployment (0.00s)

#[derive(Debug, PartialEq)]
enum Line<'g> {
    BeginFail(&'g [u8], f64),
}

named!(dash_dash_fail, tag!("--- FAIL: "));
named!(endfail, tag!("FAIL "));
named!(duration<f64>, do_parse!(val: be_f64 >> alpha1 >> (val)));
named!(delim_dur<f64>, delimited!(char!('('), duration, char!(')')));

named!(
    fail_begin<Line>,
    do_parse!(
        dash_dash_fail >> name: alphanumeric1 >> space >> dur: delim_dur >> eol
            >> (Line::BeginFail(name, dur))
    )
);

#[cfg(test)]
mod tests {
    #[test]
    fn fail_begin() {
        let input = b"--- FAIL: TestXX (0.00s)\n";

        assert_eq!(
            super::fail_begin(input),
            Ok((&b""[..], super::Line::BeginFail(&b"TestXX"[..], 0.0)))
        );
    }

    #[test]
    fn endfail_matches() {
        assert_eq!(super::endfail(b"FAIL "), Ok((&b""[..], &b"FAIL "[..])));
    }

    #[test]
    #[should_panic]
    fn endfail_nomatch() {
        super::endfail(b"something else").unwrap();
    }

    #[test]
    fn dash_dash_fail_matches() {
        assert_eq!(
            super::dash_dash_fail(b"--- FAIL: "),
            Ok((&b""[..], &b"--- FAIL: "[..]))
        );
    }

    #[test]
    #[should_panic]
    fn dash_dash_fail_nomatch() {
        super::dash_dash_fail(b"something else").unwrap();
    }
}
