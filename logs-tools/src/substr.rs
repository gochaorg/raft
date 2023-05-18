#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct CharsCount(pub usize);

impl CharsCount {
    pub fn is_zero(&self) -> bool { self.0 == 0 }
    pub fn decrase(&self) -> Option<Self> { 
        if self.0 > 0 {
            Some(Self( self.0 - 1 ))
        }else{
            None
        }
    }
}

impl std::ops::Add<CharsCount> for CharsCount {
    type Output = CharsCount;
    fn add(self, rhs: CharsCount) -> Self::Output {
        CharsCount( self.0 + rhs.0 )
    }
}


pub trait SubString 
where Self: Sized
{
    fn substring( self, offset:CharsCount ) -> Option<Self>;
}

impl SubString for &str 
where Self: Sized
{
    fn substring( self, offset:CharsCount ) -> Option<Self> {
        if offset.is_zero() {
            return Some(self);
        }

        let mut chr_iter = self.char_indices();
        let mut off = offset;
        loop {
            match chr_iter.next() {
                Some((idx,_)) => {
                    if off.is_zero() {
                        return Some( &(*self)[idx..] );
                    }else{
                        match off.decrase() {
                            Some(off1) => { 
                                off = off1; 
                            },
                            None => return None
                        }
                    }
                },
                None => {
                    if off.is_zero() {
                        return Some( "" )
                    }
                    return None
                }
            }
        }
    }
}

#[test]
fn test_substr() {
    let str = "abc".to_string();
    let str = &str[..];

    let res = str.substring(CharsCount(1));    
    println!("{:?}",res);
    assert!( res == Some("bc") );

    let res = str.substring(CharsCount(2));    
    println!("{:?}",res);
    assert!( res == Some("c") );

    let res = str.substring(CharsCount(3));
    println!("{:?}",res);
    assert!( res == Some("") );

    let res = str.substring(CharsCount(4));
    println!("{:?}",res);
    assert!( res == None );
}