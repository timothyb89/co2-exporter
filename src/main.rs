use std::sync::{Arc, RwLock};
use std::thread;

use co2mon::{Sensor, Reading};
use warp::Filter;
use serde_json::{self, json};

fn read_thread(reading_lock: Arc<RwLock<Option<Reading>>>){
  thread::spawn(move || {
    let sensor = Sensor::open_default().unwrap();

    loop {
      let r = sensor.read().unwrap();
      eprintln!("reading: {:?}", r);

      let mut latest_reading = reading_lock.write().unwrap();
      *latest_reading = Some(r);
    }
  });
}

fn c_to_f(temp: f32) -> f32 {
  temp * (9f32 / 5f32) + 32f32
}

#[tokio::main]
async fn main() {
  let latest_reading_lock = Arc::new(RwLock::new(None));

  read_thread(latest_reading_lock.clone());

  let json_lock = Arc::clone(&latest_reading_lock);
  let r_json = warp::path("json").map(move || {
    match *json_lock.read().unwrap() {
      Some(ref r) => warp::reply::json(&json!({
        "temperature_c": r.temperature(),
        "temperature_f": c_to_f(r.temperature()),
        "co2": r.co2()
      })),
      None => warp::reply::json(&json!(null))
    }
  });

  let metrics_lock = Arc::clone(&latest_reading_lock);
  let r_metrics = warp::path("metrics").map(move || {
    match *metrics_lock.read().unwrap() {
      Some(ref r) => format!(
        "temperature{{unit=\"c\"}} {}\ntemperature{{unit=\"f\"}} {}\nco2{{}} {}",
        r.temperature(),
        c_to_f(r.temperature()),
        r.co2()
      ),
      None => format!("")
    }
  });

  let routes = warp::get().and(r_json).or(r_metrics);
  warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
