//! Трекер скорости выполнения участков кода

use std::{sync::{RwLock, Arc}, collections::HashMap, time::{Duration, Instant}, fmt::Display};

#[derive(Debug, Clone)]
pub struct Tracker {
  pub prefix: String,
  pub tracks: Arc<RwLock<HashMap<String,(u64, Duration)>>>
}

impl Tracker {
  pub fn new() -> Self {
    Self {
      tracks: Arc::new(RwLock::new(HashMap::new())),
      prefix: String::new()
    }
  }

  /// Добавляет/учитывает вызов метода
  /// 
  /// Параметры
  /// - `name` - имя метода/участка кода
  /// - `duration` - продолжительность
  pub fn add( &self, name:&str, duration:Duration ) {
    let tracks = self.tracks.write();
    match tracks {
      Err(_) => {},
      Ok(mut tracks) => {
        let name = name.to_string();
        let v = match tracks.get(&name) {
          Some( (c,d) ) => {
            (c + 1, *d + duration)
          },
          None => {
            (1u64, duration)
          }
        };
        tracks.insert(format!("{p}{n}",n=name,p=self.prefix), v);
      }
    }
  }

  /// Замеряет вызов метода/участка
  /// 
  /// Параметры
  /// - `name` - имя метода/участка кода
  /// - `tracked` - замеряемый участок
  pub fn track<F,R>( &self, name:&str, tracked:F ) -> R 
  where F: FnOnce() -> R
  {
    let t0 = Instant::now();
    let res = tracked();
    let t1 = Instant::now();
    let dur = t1.duration_since(t0);
    self.add(name, dur);
    res
  }

  /// Создает новый трекер для учета вложенных/дочерных вызовов
  pub fn sub_tracker( &self, prefix: &str ) -> Self {
    let mut new_prefix = String::new();
    new_prefix.push_str(&self.prefix);
    new_prefix.push_str(prefix);
    Self { prefix: new_prefix, tracks: self.tracks.clone() }
  }

}

impl Display for Tracker {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.tracks.read() {
      Err(err) => write!(f,"tracks can't display {}", err.to_string()),
      Ok(tracks) => {
        let mut msg = String::new();
        msg.push_str("tracks:\n");
    
        let mut keys : Vec<String> = tracks.keys().into_iter().map(|s| s.clone()).collect();
        keys.sort();
        for key in keys {
          match tracks.get(&key) {
            Some( (cnt,dur) ) => {
              let avg = Duration::from_nanos( (dur.as_nanos() / (*cnt as u128)) as u64 );

              msg.push_str(&format!("{key} cnt={cnt} dur.sum={dur:?} dur.avg={avg:?}\n"))
            },
            None => {}
          }
        }
    
        write!(f, "{}", msg)
      }
    }
  }
}