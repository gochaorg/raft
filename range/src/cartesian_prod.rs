/// Декартово произведение
/// 
/// # Аргументы
/// 
/// - a - Итератор 1
/// - b - Итератор 2
/// 
/// # Пример
/// 
/// ```
/// let mut itr = product(vec![1,2,3], vec![4,5]);
/// assert_eq!( itr.next(), Some((1,4)) );
/// assert_eq!( itr.next(), Some((1,5)) );
/// assert_eq!( itr.next(), Some((2,4)) );
/// assert_eq!( itr.next(), Some((2,5)) );
/// assert_eq!( itr.next(), Some((3,4)) );
/// assert_eq!( itr.next(), Some((3,5)) );
/// ```
pub fn product<A:IntoIterator<Item=I>,B:IntoIterator<Item=II> + Clone,I:Clone,II:Clone>( a:A, b:B ) -> impl Iterator<Item = (I,II)>  {
    let r = a.into_iter().flat_map(move |av| {
        b.clone().into_iter().map(move |bv| (av.clone(),bv.clone()))
    });
    r
}

#[test]
fn product_test() {
    let mut itr = product(vec![1,2,3], vec![4,5]);
    assert_eq!( itr.next(), Some((1,4)) );
    assert_eq!( itr.next(), Some((1,5)) );
    assert_eq!( itr.next(), Some((2,4)) );
    assert_eq!( itr.next(), Some((2,5)) );
    assert_eq!( itr.next(), Some((3,4)) );
    assert_eq!( itr.next(), Some((3,5)) );
}