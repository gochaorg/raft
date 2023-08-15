use parse::{Number};
use super::{Multiple, RangeNum};
use super::range::Range;

/// Трансформация из [Multiple] (AST) в [Range]
impl<A> TryFrom<Multiple> for Range<A> 
where A: TryFrom<Number,Error=String> + Clone
{
    type Error = String;
    fn try_from(value: Multiple) -> Result<Self, Self::Error> {
        fn range_num<A>(r_n: RangeNum) -> Result<Range<A>,String> 
        where A: TryFrom<Number,Error = String> + Clone
        {
            match r_n {
                RangeNum::One(n) => {
                    let v:A = n.0.try_into()?;
                    Ok(Range::Single(v))
                },
                RangeNum::Range(f_t) => {
                    let a:A = f_t.0.try_into()?;
                    let b:A = f_t.1.try_into()?;
                    Ok(Range::FromToInc(a, b))
                }
            }
        }

        let x = value.0.into_iter()
            .map(|v| range_num(v))
            .fold(
                Ok(
                    Box::new(Vec::<Range<A>>::new())
                ), 
                |acc,itm| {
            acc.and_then(|mut acc| 
                itm.map(|itm| {
                    acc.push(itm.clone());
                    acc
                })
            )
        })?;

        Ok(Range::Multiple(*x.clone()))
    }
}

#[test]
fn multiple_parse_test() {
    use super::MultipleParse;
    use parse::*;

    let parser = MultipleParse::new();
    let res = parser.parse("1,2,4-6");
    println!("{:?}", res);

    let (res,_) = res.unwrap();

    let range : Range<u64> = res.try_into().unwrap();
    println!("{:?}", range);
}
