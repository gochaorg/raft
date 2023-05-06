use std::{collections::HashMap};

/// Метрики
/// 
/// Содержит счетчики которые монотонно увеличиваются
pub trait Metrics {
    /// Увеличивает значение конкретной метрики
    fn inc<'a,'b>( &'a mut self, name:&'b str );

    /// Создает снимок метрик
    fn snapshot( &self ) -> Self;
}

#[derive(Debug,Clone)]
pub struct DummyCounters;

impl Metrics for DummyCounters {
    fn inc<'a,'b>( &'a mut self, name:&'b str ) {        
    }

    fn snapshot( &self ) -> Self {
        self.clone()
    }
}

/// Счетчики расположенные в памяти
#[derive(Debug,Clone)]
pub struct Counters {
    pub map: Box<HashMap<String,u64>>
}

impl Counters {
    pub fn new() -> Self {
        Self { map: Box::new(HashMap::new()) }
    }
}

impl Metrics for Counters {
    fn inc<'a,'b>( &'a mut self, name:&'b str ) {
        self.map.insert(name.to_string(), 
            match self.map.get(&name.to_string()) {
                Some(v) => *v + 1,
                None => 1u64
            }
        );        
    }

    fn snapshot( &self ) -> Self {
        self.clone()
    }
}

/// Итератор по значениями через ссылку
pub struct CountersItr<'a> {
    iter: std::collections::hash_map::Iter<'a,String,u64>
}

impl<'a> Iterator for CountersItr<'a> {
    type Item = (String,u64);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k,v)| (k.clone(), v.clone()))
    }
}

impl<'a> IntoIterator for &'a Counters {
    type IntoIter = CountersItr<'a>;
    type Item = (String,u64);

    fn into_iter(self) -> Self::IntoIter {
        let it = self.map.iter();
        CountersItr {
            iter: it
        }
    }
}

/// Итератор по копии счетчиков
/// 
/// Названия метрик отсортированы
pub struct CountersItrBySnapshot {
    counters: Counters,
    keys: Box<Vec<String>>,
    pointer: usize
}

impl From<Counters> for CountersItrBySnapshot {
    fn from(value: Counters) -> Self {
        let mut keys:Box<Vec<String>> = Box::new(value.map.keys().map(|k|k.clone()).collect());
        keys.sort();
        Self {
            counters: value.clone(),
            keys: keys,
            pointer: 0
        }
    }
}

impl Iterator for CountersItrBySnapshot {
    type Item = (String,u64);
    fn next(&mut self) -> Option<Self::Item> {
        if self.pointer > self.keys.len() {
            None
        } else {
            let k = self.keys[self.pointer].clone();
            let v = self.counters.map[&k];
            self.pointer += 1;
            Some((k,v))
        }
    }
}

impl IntoIterator for Counters {
    type IntoIter = CountersItrBySnapshot;
    type Item = (String, u64);
    fn into_iter(self) -> Self::IntoIter {
        CountersItrBySnapshot::from(self)
    }
}

#[test]
fn test_counters() {
    let mut counters = Counters::new();
    counters.inc("name1");
    counters.inc("name1");
    counters.inc("name2");
    
    for (name,cnt) in &counters {
        println!("{name}={cnt}")
    }
}