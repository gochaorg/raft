use crate::parse::Number;
use super::{Multiple, RangeNum};
use super::range::Range;

impl TryFrom<Multiple> for Range<u128> 
{
    type Error = String;
    fn try_from(value: Multiple) -> Result<Self, Self::Error> {
        fn range_num
        (r_n: RangeNum) -> Result<Range<u128>,String> {
            match r_n {
                RangeNum::One(n) => {
                    let v:u128 = n.0.try_into()?;
                    Ok(Range::Single(v))
                },
                RangeNum::Range(f_t) => {
                    let a:u128 = f_t.0.try_into()?;
                    let b:u128 = f_t.1.try_into()?;
                    Ok(Range::FromToInc(a, b))
                }
            }
        }

        let x = value.0.into_iter()
            .map(|v| range_num(v))
            .fold(
                Ok(
                    Box::new(Vec::<Range<u128>>::new())
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