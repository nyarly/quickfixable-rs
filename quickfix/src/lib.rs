extern crate failure;
#[macro_use]
extern crate nom;

use failure::Error;

// xxx pub
pub fn filter(i: &[u8]) -> Result<&[u8], Error> {
    Ok(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::from_utf8;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn go_one() {
        let input = include_bytes!("../testdata/one/in.txt");
        let filtered = filter(input).unwrap();
        let output = include_bytes!("../testdata/one/out.txt");
        assert_eq!(from_utf8(filtered), from_utf8(output))
    }
}
// xxx pub
pub mod parse;
