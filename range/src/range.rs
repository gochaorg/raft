/// Хранит диапазоны значений
/// 
/// Например для `1,2,4,8-10` будет такая структура:
/// 
/// ```
/// Multiple(
///   Single(1),
///   Single(2),
///   Single(4),
///   FromToInc(8,10),
/// )
/// ```
/// 
/// По данной структуре возможно итерироваться:
/// 
/// ```
/// let range = Range::Multiple( vec![
///     Range::Single(1u32),
///     Range::FromToExc(7,10),
///     Range::Multiple( vec![
///         Range::FromToInc(12,15)
///     ])
/// ]);
/// let mut iter = range.into_iter();
/// for _ in 0..10 { 
///     let v = iter.next();
///     println!("{:?}",v);
/// }
/// ```
#[derive(Clone,Debug)]
#[allow(dead_code)]
pub enum Range<T:Sized+Clone> {
    Single(T),
    FromToExc(T,T),
    FromToInc(T,T),
    Multiple(Vec<Range<T>>)
}

pub trait Next<T:Sized> {
    fn next(v:T) -> Option<T>;
}

impl Next<u64> for u64 {
    fn next(v:u64) -> Option<u64> {
        if v == u64::MAX {
            None
        } else {
            Some(v+1)
        }
    }
}

impl Next<u32> for u32 {
    fn next(v:u32) -> Option<u32> {
        if v == u32::MAX {
            None
        } else {
            Some(v+1)
        }
    }
}

impl Next<i32> for i32 {
    fn next(v:i32) -> Option<i32> {
        if v == i32::MAX {
            None
        } else {
            Some(v+1)
        }
    }
}

impl Next<u16> for u16 {
    fn next(v:u16) -> Option<u16> {
        if v == u16::MAX {
            None
        } else {
            Some(v+1)
        }
    }
}

impl Next<u8> for u8 {
    fn next(v:u8) -> Option<u8> {
        if v == u8::MAX {
            None
        } else {
            Some(v+1)
        }
    }
}

struct SingleIter<T:Clone> {
    value: T,
    ptr: u8
}

impl<T:Clone> SingleIter<T> {
    pub fn new( v: &T ) -> Self {
        Self { value: v.clone(), ptr: 0 }
    }
}

impl<T:Clone> Iterator for SingleIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr == 0 { 
            self.ptr += 1;
            Some(self.value.clone()) 
        } else {
            None
        }
    }
}

pub struct FromToIter<T:Clone+Next<T>+Eq+Ord> {
    pub to:T,
    pub cur:T,
    pub include_to:bool,
    pub finished:bool,
}

impl<T:Clone+Next<T>+Eq+Ord> Iterator for FromToIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished { return None }

        let has_next = if self.include_to { self.cur <= self.to } else { self.cur < self.to };

        if has_next {
            let res = self.cur.clone();
            
            let next_opt = <T as Next::<T>>::next(self.cur.clone());

            match next_opt {
                Some(nxt) => {
                    self.cur = nxt.clone();
                },
                None => {
                    self.finished = true;
                }
            };

            Some(res)
        } else {
            None
        }
    }
}

#[test]
fn test_from_to() {
    let mut itr = FromToIter {
        to: 3u32,
        include_to: false,
        cur: 0u32,
        finished: false
    };

    println!("{:?}", itr.next());
    println!("{:?}", itr.next());
    println!("{:?}", itr.next());
    println!("{:?}", itr.next());
    println!("{:?}", itr.next());
}

struct IterOnVecIter<C,T> 
where
    C: IntoIterator<Item = T> + Clone
{
    pub vec: Vec<C>,
    pub cur_item: usize,
    pub cur_iter: Option<Box<dyn Iterator<Item = T>>>
}

impl<C,T> Iterator for IterOnVecIter<C,T> 
where
    C: IntoIterator<Item = T> + Clone + 'static
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_iter.is_some() {
            let itr = self.cur_iter.as_mut().unwrap();
            match itr.next() {
                Some(res) => return Some(res),
                None => {
                    self.cur_iter = None;
                    return self.next();
                }
            }
        }

        if self.cur_item >= self.vec.len() {
            return None
        }

        let coll = self.vec[self.cur_item].clone().into_iter();
        let coll1 = Box::new( coll ) as Box<dyn Iterator<Item=T>>;
        self.cur_iter = Some(coll1);
        self.cur_item += 1;

        self.next()
    }
}

#[test]
fn test_iter_on_vec() {
    let mut iter = IterOnVecIter {
        vec: vec![ vec![1,2], vec![3,4] ],
        cur_item: 0,
        cur_iter: None
    };

    println!( "{:?}", iter.next() );
    println!( "{:?}", iter.next() );
    println!( "{:?}", iter.next() );
    println!( "{:?}", iter.next() );
    println!( "{:?}", iter.next() );
}

impl<T> Range<T> 
where
    T: Clone+Sized+Next<T>+Ord+'static
{
    pub fn iter( &self ) -> Box<dyn Iterator<Item = T>> {
        match self {
            Self::Single(v) => {
                Box::new( SingleIter::new(v) ) 
            },
            Self::FromToExc(from, to) => {
                Box::new( FromToIter {
                    to: to.clone(),
                    cur: from.clone(),
                    include_to: false,
                    finished: false
                }) 
            },
            Self::FromToInc(from, to) => {
                Box::new( FromToIter {
                    to: to.clone(),
                    cur: from.clone(),
                    include_to: true,
                    finished: false
                }) 
            },
            Self::Multiple(items) => {
                let itm = items.to_vec();
                Box::new( IterOnVecIter {
                    vec: itm,
                    cur_item: 0,
                    cur_iter: None
                })
            }
        }
    }
}

impl<T> IntoIterator for Range<T> 
where
    T: Clone+Sized+Next<T>+Ord+'static
{
    type Item = T;
    type IntoIter = Box<dyn Iterator<Item = T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[test]
fn range_iter_test() {
    let range = Range::Single(10);
    let mut iter = range.into_iter();
    //for _ in 0..5 { println!("{:?}",iter.next()); }
    assert_eq!( iter.next(), Some(10) );
    assert_eq!( iter.next(), None );

    let range = Range::FromToExc(7,10);
    let mut iter = range.into_iter();
    //for _ in 0..5 { println!("{:?}",iter.next()); }
    assert_eq!( iter.next(), Some(7) );
    assert_eq!( iter.next(), Some(8) );
    assert_eq!( iter.next(), Some(9) );
    assert_eq!( iter.next(), None );
    assert_eq!( iter.next(), None );

    let range = Range::Multiple( vec![
        Range::Single(1),
        Range::FromToExc(7,10)
    ]);
    let mut iter = range.into_iter();
    assert_eq!( iter.next(), Some(1) );
    assert_eq!( iter.next(), Some(7) );
    assert_eq!( iter.next(), Some(8) );
    assert_eq!( iter.next(), Some(9) );
    assert_eq!( iter.next(), None );

    let range = Range::Multiple( vec![
        Range::Single(1u32),
        Range::FromToExc(7,10),
        Range::Multiple( vec![
            Range::FromToInc(12,15)
        ])
    ]);
    let mut iter = range.into_iter();
    assert_eq!( iter.next(), Some(1u32) );
    assert_eq!( iter.next(), Some(7u32) );
    assert_eq!( iter.next(), Some(8u32) );
    assert_eq!( iter.next(), Some(9u32) );
    assert_eq!( iter.next(), Some(12u32) );
    assert_eq!( iter.next(), Some(13u32) );
    assert_eq!( iter.next(), Some(14u32) );
    assert_eq!( iter.next(), Some(15u32) );
    assert_eq!( iter.next(), None );
}