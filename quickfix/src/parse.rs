use nom::{digit, double, eol, is_space, space, alpha1, alphanumeric1, hex_u32};

#[derive(Debug, PartialEq)]
pub enum Line<'g> {
    BeginFail(&'g [u8], f64),
    FailedPackage(&'g [u8], f64),
    PanicStart(&'g [u8]),
    PanicCause(&'g [u8]),
    StackStart(i64),
    StackFunc(&'g [u8], Vec<&'g [u8]>),
    StackLine(&'g [u8], i64, u32),
    Basic(&'g [u8]),
}

named!(pub golines<Vec<Line>>, many0!(goline));

named!(
    goline<Line>,
    alt!(
        fail_begin | failed_package | panic_start | panic_cause | stack_start | stack_function
            | stack_line | a_line
    )
);

//--- FAIL: TestStableDeployment (0.00s)
named!(
    fail_begin<Line>,
    dbg_dmp!(do_parse!(
        dash_dash_fail >> name: alphanumeric1 >> space >> dur: delim_dur >> eol
            >> (Line::BeginFail(name, dur))
    ))
);

// FAIL	github.com/opentable/sous/ext/singularity	0.011s
named!(
    failed_package<Line>,
    do_parse!(
        endfail >> name: take_till!(is_space) >> space >> dur: duration >> eol
            >> (Line::FailedPackage(name, dur))
    )
);

// panic: runtime error: invalid memory address or nil pointer dereference [recovered]
named!(
    panic_start<Line>,
    do_parse!(
        desc: delimited!(panic_tag, take_until!(" [recovered]"), recovered)
            >> (Line::PanicStart(desc))
    )
);

// goroutine 24 [running]:
named!(
    stack_start<Line>,
    do_parse!(
        tag!("goroutine ") >> grn: integer >> tag!(" [running]:") >> eol >> (Line::StackStart(grn))
    )
);

// [signal SIGSEGV: segmentation violation code=0x1 addr=0x4c pc=0x90e072]
named!(
    panic_cause<Line>,
    do_parse!(
        cause: delimited!(char!('['), is_not!("]"), char!(']')) >> eol >> (Line::PanicCause(cause))
    )
);

// testing.tRunner.func1(0xc4203a3520)

named!(
    stack_function<Line>,
    do_parse!(
        fname: is_not!("(")
            >> parms:
                delimited!(
                    char!('('),
                    separated_list!(ws!(char!(',')), is_not!(",)")),
                    char!(')')
                ) >> eol >> (Line::StackFunc(fname, parms))
    )
);

// 	/home/judson/golang/src/github.com/opentable/sous/ext/singularity/deployer_test.go:293 +0x5bc
named!(
    stack_line<Line>,
    do_parse!(
        space >> file: is_not!(":") >> char!(':') >> line: integer >> tag!("+0x") >> offset: hex_u32
            >> (Line::StackLine(file, line, offset))
    )
);

// github.com/opentable/sous/ext/singularity.(*deploymentBuilder).unpackDeployConfig(0xc4203aefc0, 0xc4203a3520, 0x0)
// named!(stack_method, ...);

named!(
    a_line<Line>,
    do_parse!(line: rest_of_line >> (Line::Basic(line)))
);

named!(dash_dash_fail, tag!("--- FAIL: "));
named!(endfail, tag!("FAIL\t"));
named!(duration<f64>, do_parse!(val: double >> alpha1 >> (val)));
named!(delim_dur<f64>, delimited!(char!('('), duration, char!(')')));
named!(rest_of_line, is_not!("\n\r"));

named!(panic_tag, tag!("panic: "));
named!(recovered, tag!(" [recovered]"));

use std::str::{self, FromStr};
named!(
    integer<i64>,
    map_res!(map_res!(ws!(digit), str::from_utf8), FromStr::from_str)
);

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use nom::IResult;
    fn parse_result<T>(res: IResult<&[u8], T>, should_match: T)
    where
        T: PartialEq + Debug,
    {
        use nom::Err::{Error, Incomplete};
        use nom::Context::Code;
        use nom::Needed::Size;
        use std::string::String;
        match res {
            Ok((_, did_match)) => assert_eq!(should_match, did_match),
            Err(Error(Code(code, err))) => {
                panic!(format!(
                    "Error {:?}:'{}'",
                    err,
                    String::from_utf8_lossy(code)
                ));
            }
            Err(Incomplete(Size(n))) => panic!(format!("Incomplete: needs {} more bytes.", n)),
            _ => panic!(format!("{:?}", res)),
        }
    }

    #[test]
    fn fail_begin() {
        parse_result(
            super::fail_begin(b"--- FAIL: TestXX (0.00s)\n"),
            super::Line::BeginFail(&b"TestXX"[..], 0.0),
        );

        parse_result(
            super::fail_begin(b"--- FAIL: TestPi (3.14s)\n"),
            super::Line::BeginFail(&b"TestPi"[..], 3.14),
        );
    }

    #[test]
    fn failed_package() {
        parse_result(
            super::failed_package(b"FAIL	github.com/opentable/sous/ext/singularity	0.011s\n"),
            super::Line::FailedPackage(&b"github.com/opentable/sous/ext/singularity"[..], 0.011),
        )
    }

    #[test]
    fn panic_start() {
        parse_result(
        super::panic_start(b"panic: runtime error: invalid memory address or nil pointer dereference [recovered]\n"),
        super::Line::PanicStart(&b"runtime error: invalid memory address or nil pointer dereference"[..]),
        )
    }

    #[test]
    fn panic_cause() {
        parse_result(
            super::panic_cause(
                b"[signal SIGSEGV: segmentation violation code=0x1 addr=0x4c pc=0x90e072]\n",
            ),
            super::Line::PanicCause(
                &b"signal SIGSEGV: segmentation violation code=0x1 addr=0x4c pc=0x90e072"[..],
            ),
        )
    }

    #[test]
    fn stack_function() {
        parse_result(
            super::stack_function(b"testing.tRunner.func1(0xc4203a3520)\n"),
            super::Line::StackFunc(&b"testing.tRunner.func1"[..], vec![&b"0xc4203a3520"[..]]),
        )
    }

    #[test]
    fn stack_line() {
        parse_result(
            super::stack_line(b" /home/judson/golang/src/github.com/opentable/sous/ext/singularity/deployer_test.go:293 +0x5bc\n"),
            super::Line::StackLine(&b"/home/judson/golang/src/github.com/opentable/sous/ext/singularity/deployer_test.go"[..], 293, 1468),
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
