use std::{sync::{RwLock, Arc}, collections::HashMap, time::{Duration, Instant}, fmt::Display};

pub struct Tracker {
  pub tracks: Arc<RwLock<HashMap<String,(u64, Duration)>>>
}

impl Tracker {
  fn track<F,R>( &self, name:&str, tracked:F ) -> R 
  where F: FnOnce() -> R
  {
    let t0 = Instant::now();
    let res = tracked();
    let t1 = Instant::now();
    let dur = t1.duration_since(t0);
    {
      let mut tracks = self.tracks.write();
      match tracks {
        Err(_) => {},
        Ok(mut tracks) => {
          let name = name.to_string();
          let v = match tracks.get(&name) {
            Some( (c,d) ) => {
              (c + 1, *d + dur)
            },
            None => {
              (1u64, dur)
            }
          };
          tracks.insert(name, v);
        }
      }
    }
    res
  }
}

impl Display for Tracker {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.tracks.read() {
      Err(err) => write!(f,"tracks can't display {}", err.to_string()),
      Ok(tracks) => {
        let mut msg = String::new();
        msg.push_str("tracks:\n");
    
        let keys : Vec<String> = tracks.keys().into_iter().map(|s| s.clone()).collect();
        for key in keys {
          match tracks.get(&key) {
            Some( (cnt,dur) ) => {
              msg.push_str(&format!("{key} {cnt} {dur:?}\n"))
            },
            None => {}
          }
        }
    
        write!(f, "{}", msg)
      }
    }
  }
}