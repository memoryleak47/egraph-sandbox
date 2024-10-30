use std::collections::HashMap;
// use std::sync::atomic::AtomicUsize;
use std::time::{Instant, Duration};
use std::sync::RwLock;
use std::ops::DerefMut;

// TODO: use tracing_subscriber Registry/Layers/Filters for modularity and multi-threading?

pub struct BucketSubscriber {
  lock: RwLock<AllData>
}

pub struct AllData {
  buckets: HashMap<tracing::Id, SpanData>,
  entered_count: u64,
}

pub struct SpanData {
  name: &'static str,
  enter_time: Option<Instant>,
  total_time: Duration,
  ref_count: u64,
  entered_count: u32,
}

impl Default for BucketSubscriber {
  fn default() -> Self {
    BucketSubscriber::new()
  }
}

impl BucketSubscriber {
  pub fn new() -> Self {
    Self { lock: RwLock::new(AllData {
      buckets: HashMap::new(),
      entered_count: 0,
    })}
  }
}

impl AllData {
  fn display(buckets: &HashMap<tracing::Id, SpanData>) {
    let mut sorted_data: Vec<&SpanData> = buckets.values().collect();
    sorted_data.sort_by(|a, b| b.total_time.cmp(&a.total_time));
    let root_time_secs = sorted_data[0].total_time.as_secs_f64();
    for data in sorted_data {
      let percentage = 100.0 * data.total_time.as_secs_f64() / root_time_secs;
      println!("{}: {:.2?} ({:.2}%, {} calls)", data.name, data.total_time, percentage, data.ref_count);
      if data.enter_time.is_some() {
        eprintln!("bug: enter_time is some at display");
      }
      if data.entered_count > 0 {
        eprintln!("bug: entered_count > 0 at display");
      }
    }
  }
}

impl tracing::Subscriber for BucketSubscriber {
  fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
    // TODO: could filter some things we don't always care about here
    metadata.is_span() // for now, only subscribe to spans, not events
  }

  fn new_span(&self, attrs: &tracing::span::Attributes<'_>) -> tracing::Id {
    let name: &'static str = attrs.metadata().name();
    let id = tracing::Id::from_u64(name.as_ptr() as usize as u64);
    self.lock.write().unwrap().buckets.entry(id.clone()).or_insert(SpanData {
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

  fn event(&self, _event: &tracing::Event<'_>) {
    ()
  }

  fn enter(&self, span: &tracing::Id) {
    let mut all_data = self.lock.write().unwrap();
    let data = all_data.buckets.get_mut(span)
      .expect("span not found, this is a bug");
    if data.entered_count > 0 {
      // this is a nested call, keep outer time
    } else {
      data.enter_time = Some(Instant::now());
    }
    data.ref_count += 1;
    data.entered_count += 1;
    all_data.entered_count += 1;
  }

  fn exit(&self, span: &tracing::Id) {
    let mut all_data = self.lock.write().unwrap();
    let AllData { buckets, entered_count } = all_data.deref_mut();
    *entered_count -= 1;
    let data = buckets.get_mut(span)
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
    if *entered_count == 0 {
      // this is the root
      println!("<END TRACING: {}>", data.name);
      AllData::display(buckets);
    }
  }
}