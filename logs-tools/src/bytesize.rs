use lazy_regex::{regex, Lazy};
use regex::Regex;

#[derive(Debug, Clone, Copy)]
pub struct ByteSize(pub usize);

static PLAIN: &Lazy<Regex> = regex!(r##"^(?<num>\d+)$"##);
static KB: &Lazy<Regex> = regex!(r##"^(?i)(?<num>\d+)\s*kb?$"##);
static MB: &Lazy<Regex> = regex!(r##"^(?i)(?<num>\d+)\s*mb?$"##);
static GB: &Lazy<Regex> = regex!(r##"^(?i)(?<num>\d+)\s*gb?$"##);

impl ByteSize {
    pub fn parse(text: &str) -> Result<ByteSize, String> {
        let plain = PLAIN
            .captures(text)
            .map(|c| c.name("num").unwrap().as_str())
            .map(|s| s.parse::<usize>().unwrap());

        let kb = KB
            .captures(text)
            .map(|c| c.name("num").unwrap().as_str())
            .map(|s| s.parse::<usize>().unwrap() * 1024);

        let mb = MB
            .captures(text)
            .map(|c| c.name("num").unwrap().as_str())
            .map(|s| s.parse::<usize>().unwrap() * 1024 * 1024);

        let gb = GB
            .captures(text)
            .map(|c| c.name("num").unwrap().as_str())
            .map(|s| s.parse::<usize>().unwrap() * 1024 * 1024 * 1024);

        let size = plain.or(kb).or(mb).or(gb).map(|s| ByteSize(s));

        size.ok_or("can't parse size".to_string())
    }
}

impl PartialEq for ByteSize {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for ByteSize {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

#[test]
fn test_parse() {
    println!("{:?}", ByteSize::parse("123"));
    println!("{:?}", ByteSize::parse("12 k"));
    println!("{:?}", ByteSize::parse("13k"));
    println!("{:?}", ByteSize::parse("14K"));
    println!("{:?}", ByteSize::parse("15Kb"));
    println!("{:?}", ByteSize::parse("15m"));
}
