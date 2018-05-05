#![allow(non_snake_case)]

use nom::{double, eol, is_space, space, alpha1, alphanumeric1};

#[derive(Debug, PartialEq)]
pub enum Line<'g> {
    BeginFail(&'g [u8], f64),
    FailedPackage(&'g [u8], f64),
}

named!(dash_dash_fail, tag!("--- FAIL: "));
named!(endfail, tag!("FAIL\t"));
named!(duration<f64>, do_parse!(val: double >> alpha1 >> (val)));
named!(delim_dur<f64>, delimited!(char!('('), duration, char!(')')));

//--- FAIL: TestStableDeployment (0.00s)
named!(
    pub fail_begin<Line>,
    dbg_dmp!(do_parse!(
        dash_dash_fail >> name: alphanumeric1 >> space >> dur: delim_dur >> eol
            >> (Line::BeginFail(name, dur))
    ))
);

// FAIL	github.com/opentable/sous/ext/singularity	0.011s
named!(
    pub failed_package<Line>,
    do_parse!(
        endfail >> name: take_till!(is_space) >> space >> dur: duration >> eol
            >> (Line::FailedPackage(name, dur))
    )
);

#[cfg(test)]
mod tests {
    #[test]
    fn fail_begin() {
        assert_eq!(
            super::fail_begin(b"--- FAIL: TestXX (0.00s)\n"),
            Ok((&b""[..], super::Line::BeginFail(&b"TestXX"[..], 0.0)))
        );

        assert_eq!(
            super::fail_begin(b"--- FAIL: TestPi (3.14s)\n"),
            Ok((&b""[..], super::Line::BeginFail(&b"TestPi"[..], 3.14)))
        );
    }

    #[test]
    fn failed_package() {
        use nom::Err::{self, Error};
        use nom::Context::Code;
        use std::string::String;
        let res = super::failed_package(b"FAIL	github.com/opentable/sous/ext/singularity	0.011s\n");
        if let Err(Error(Code(code, err))) = res {
            println!("Error {:?}:'{}'", err, String::from_utf8_lossy(code));
        }
        assert_eq!(
            super::failed_package(b"FAIL	github.com/opentable/sous/ext/singularity	0.011s\n"),
            Ok((
                &b""[..],
                super::Line::FailedPackage(
                    &b"github.com/opentable/sous/ext/singularity"[..],
                    0.011
                )
            ))
        )
    }

    #[test]
    fn delim_dur() {
        assert_eq!(super::delim_dur(b"(0.00s)"), Ok((&b""[..], 0.0)))
    }

    #[test]
    fn endfail_matches() {
        assert_eq!(super::endfail(b"FAIL\t"), Ok((&b""[..], &b"FAIL\t"[..])));
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
