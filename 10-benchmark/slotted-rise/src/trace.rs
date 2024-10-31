use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::sync::RwLock;
use thread_local::ThreadLocal;
use std::cell::RefCell;

// TODO: use tracing_subscriber Registry/Layers/Filters for modularity?

pub struct BucketSubscriber {
  // RwLock allows mutable access to all threads data after a parallel tracing section is over
  // ThreadLocal allows each thread to aggregate its own statistics with sequential invariants
  // RefCell allows each thread mutable access to its own data
  lock: RwLock<ThreadLocal<RefCell<ThreadData>>>,
}

pub struct ThreadData {
  buckets: HashMap<tracing::Id, SpanData>,
}

pub struct SpanData {
  name: &'static str,
  enter_time: Option<Instant>,
  total_time: Duration,
  ref_count: u64,
  entered_count: u32,
}

struct SpanStats {
  name: &'static str,
  total_time: Duration,
  ref_count: u64,
}

impl Default for BucketSubscriber {
  fn default() -> Self {
    BucketSubscriber::new()
  }
}

impl BucketSubscriber {
  pub fn new() -> Self {
    Self { lock: RwLock::new(ThreadLocal::new()) }
  }

  fn collect_all_thread_stats(&self) -> HashMap<tracing::Id, SpanStats> {
    let mut all_stats = HashMap::<tracing::Id, SpanStats>::new();
    for thread_data in self.lock.try_write().unwrap().iter_mut() {
      for (id, data) in &mut thread_data.get_mut().buckets.drain() {
        if data.enter_time.is_some() {
          eprintln!("bug: enter_time is some at collection time");
        }
        if data.entered_count > 0 {
          eprintln!("bug: entered_count > 0 at collection time");
        }
        all_stats.entry(id.clone())
          .and_modify(|d| {
            d.total_time = d.total_time.saturating_add(data.total_time);
            if d.total_time == Duration::MAX {
              eprintln!("bug: overflowed on {} time", data.name);
            }
            d.ref_count += data.ref_count;
          })
          .or_insert(SpanStats {
            name: data.name,
            total_time: data.total_time,
            ref_count: data.ref_count,
          });
      }
    }
    all_stats
  }

  fn display(&self) {
    println!("<DISPLAYING TRACED DATA>");

    let all_stats = self.collect_all_thread_stats();

    let mut sorted_data: Vec<&SpanStats> = all_stats.values().collect();
    sorted_data.sort_by(|a, b| b.total_time.cmp(&a.total_time));
    let root_time_secs = sorted_data[0].total_time.as_secs_f64();
    for data in sorted_data {
      let percentage = 100.0 * data.total_time.as_secs_f64() / root_time_secs;
      println!("{}: {:.2?} ({:.2}%, {} calls)", data.name, data.total_time, percentage, data.ref_count);
    }
  }
}

impl tracing::Subscriber for BucketSubscriber {
  fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
    // TODO: could filter more things here depending on use-case
    // for now, only subscribe to spans, and the special display event
    metadata.is_span() || metadata.name() == "display"
  }

  fn new_span(&self, attrs: &tracing::span::Attributes<'_>) -> tracing::Id {
    let name: &'static str = attrs.metadata().name();
    let id = tracing::Id::from_u64(name.as_ptr() as usize as u64);
    let guard = self.lock.try_read().unwrap();
    let buckets = &mut guard
      .get_or(|| RefCell::new(ThreadData { buckets: HashMap::new() }))
      .borrow_mut().buckets;
    buckets.entry(id.clone()).or_insert(SpanData {
      name,
      enter_time: None,
      total_time: Duration::ZERO,
      ref_count: 0,
      entered_count: 0,
    });
    id
  }

  fn record(&self, _span: &tracing::Id, _values: &tracing::span::Record<'_>) {
    ()
  }

  fn record_follows_from(&self, _span: &tracing::Id, _follows: &tracing::Id) {
    ()
  }

  fn event(&self, event: &tracing::Event<'_>) {
    if event.metadata().name() == "display" {
      self.display();
    }
  }

  fn enter(&self, span: &tracing::Id) {
    let guard = self.lock.try_read().unwrap();
    let mut thread_data = guard
      .get().expect("expected thread data")
      .borrow_mut();
    let data = thread_data.buckets.get_mut(span)
      .expect("span not found, this is a bug");
    if data.entered_count > 0 {
      // this is a nested call, keep outer time
    } else {
      data.enter_time = Some(Instant::now());
    }
    data.ref_count += 1;
    data.entered_count += 1;
  }

  fn exit(&self, span: &tracing::Id) {
    let guard = self.lock.try_read().unwrap();
    let mut thread_data = guard
      .get().expect("expected thread data")
      .borrow_mut();
    let data = thread_data.buckets.get_mut(span)
      .expect("span not found, this is a bug");
    data.entered_count -= 1;
    if data.entered_count > 0 {
      // this is a nested call, do not accumulate into total time
    } else {
      let exit_time = Instant::now();
      let enter_time = data.enter_time.take().expect("enter_time not found, this is a bug");
      let total_time = data.total_time.saturating_add(
        exit_time - enter_time
      );
      if total_time == Duration::MAX {
        eprintln!("bug: overflowed on {} time", data.name);
      }
      data.total_time = total_time;
    }
  }
}