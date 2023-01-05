use std::io;

pub struct StringParser {
    data: Vec<char>,
    position: usize,
}

impl StringParser {
    pub fn new(content: String) -> StringParser {
        StringParser {
            data: content.chars().collect(),
            position: 0,
        }
    }

    pub fn has_more(&self) -> bool {
        self.position < self.data.len()
    }

    pub fn n_chars(&mut self, chars: usize) -> Result<String, io::Error> {
        if self.position + chars > self.data.len() {
            return Err(io::Error::new(io::ErrorKind::Other, "End of string"));
        }

        let str = self.data[self.position..self.position + chars].iter().collect();
        self.position += chars;
        return Ok(str);
    }

    pub fn next(&mut self) -> Result<char, io::Error> {
        if self.position >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::Other, "End of string"));
        }

        let c = self.data[self.position];
        self.position += 1;
        return Ok(c);
    }

    pub fn peek(&self) -> Result<char, io::Error> {
        if self.position >= self.data.len() {
            return Err(io::Error::new(io::ErrorKind::Other, "End of string"));
        }

        return Ok(self.data[self.position]);
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }

    pub fn peek_line(&mut self) -> Result<String, io::Error> {
        let old_position = self.position;
        let maybe_line = self.next_line();
        self.position = old_position;
        return maybe_line;
    }

    pub fn until(&mut self, c: char) -> Result<String, io::Error> {
        let mut result = String::new();
        loop {
            if !self.has_more() {
                break;
            }

            let next = self.next()?;
            if next == c {
                break;
            }
            result.push(next);
        }
        return Ok(result);
    }

    pub fn next_line(&mut self) -> Result<String, io::Error> {
        let mut result = String::new();
        let mut has_cr = false;

        loop {
            if !self.has_more() {
                if has_cr {
                    result.push('\r');
                }

                break;
            }

            let next = self.next()?;
            if next == '\r' {
                if has_cr {
                    result.push('\r');
                }

                has_cr = true;
                continue;
            } else if next == '\n' {
                if has_cr {
                    break;
                }

                result.push('\n');
            } else {
                if has_cr {
                    result.push('\r');
                }

                result.push(next);
            }

            has_cr = false;
        }

        return Ok(result);
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use crate::utils;

    #[test]
    fn positive_tests() {
        let mut content = "Hello, world!";
        let mut parser = utils::StringParser::new(content.to_string());
        assert_eq!(parser.next().unwrap(), 'H');
        assert_eq!(parser.next_line().unwrap(), "ello, world!");

        content = "Hello world!\r\nWhat is going on?\r\nSomething";
        parser = utils::StringParser::new(content.to_string());
        assert_eq!(parser.next_line().unwrap(), "Hello world!");
        assert_eq!(parser.next_line().unwrap(), "What is going on?");
        assert_eq!(parser.next_line().unwrap(), "Something");

        content = "Hello world!\r\r\nWhat is going on?\rab\r\n\rSomething";
        parser = utils::StringParser::new(content.to_string());
        assert_eq!(parser.next_line().unwrap(), "Hello world!\r");
        assert_eq!(parser.next_line().unwrap(), "What is going on?\rab");
        assert_eq!(parser.next_line().unwrap(), "\rSomething");

        content = "{1:asdf}{2:asdafaae}{3:asdf";
        parser = utils::StringParser::new(content.to_string());
        assert_eq!(parser.until('}').unwrap(), "{1:asdf");
        assert_eq!(parser.until('}').unwrap(), "{2:asdafaae");
        assert_eq!(parser.until('}').unwrap(), "{3:asdf");

        content = "";
        parser = utils::StringParser::new(content.to_string());
        assert_eq!(parser.next_line().unwrap(), "");
        assert_eq!(parser.until(' ').unwrap(), "");

        content = "Random text";
        parser = utils::StringParser::new(content.to_string());
        assert_eq!(parser.until('$').unwrap(), "Random text");

        content = "Random text";
        parser = utils::StringParser::new(content.to_string());
        assert_eq!(parser.next_line().unwrap(), "Random text");

        content = "ABCD";
        parser = utils::StringParser::new(content.to_string());

        assert_eq!(parser.has_more(), true);
        parser.next().unwrap();
        assert_eq!(parser.has_more(), true);
        parser.next_line().unwrap();
        assert_eq!(parser.has_more(), false);
    }

    #[test]
    fn negative_tests() {
        let mut content = "";
        let mut parser = utils::StringParser::new(content.to_string());

        assert_eq!(parser.next().unwrap_err().kind(), io::ErrorKind::Other);

        content = "Abcd";
        parser = utils::StringParser::new(content.to_string());

        parser.next_line().unwrap();
        assert_eq!(parser.next().unwrap_err().kind(), io::ErrorKind::Other);
    }
}