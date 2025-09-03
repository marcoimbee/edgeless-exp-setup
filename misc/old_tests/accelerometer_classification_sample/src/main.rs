// Simulation of accelerometric data
// <batch_size> samples are generated before sleeping <interval_seconds>

use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
struct AccelerometerData {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug)]
struct Features {
    mean_x: f64,
    mean_y: f64,
    mean_z: f64,
    var_x: f64,
    var_y: f64,
    var_z: f64,
}

#[derive(Debug)]
enum Classification {
    LowActivity,
    HighActivity,
    Unknown,
}

fn generate_sample() -> AccelerometerData {
    let mut rng = rand::thread_rng();
    AccelerometerData {
        x: rng.gen_range(-10.0..10.0),
        y: rng.gen_range(-10.0..10.0),
        z: rng.gen_range(-10.0..10.0),
    }
}

fn generate_batch(batch_size: usize) -> Vec<AccelerometerData> {
    (0..batch_size).map(|_| generate_sample()).collect()
}

fn extract_features(batch: &[AccelerometerData]) -> Features {
    let batch_size = batch.len() as f64;
    let (sum_x, sum_y, sum_z): (f64, f64, f64) = batch.iter().fold((0.0, 0.0, 0.0), |acc, data| {
        (acc.0 + data.x, acc.1 + data.y, acc.2 + data.z)
    });

    let mean_x = sum_x / batch_size;
    let mean_y = sum_y / batch_size;
    let mean_z = sum_z / batch_size;

    let (var_x, var_y, var_z): (f64, f64, f64) = batch.iter().fold((0.0, 0.0, 0.0), |acc, data| {
        (
            acc.0 + (data.x - mean_x).powi(2),
            acc.1 + (data.y - mean_y).powi(2),
            acc.2 + (data.z - mean_z).powi(2),
        )
    });

    Features {
        mean_x,
        mean_y,
        mean_z,
        var_x: var_x / batch_size,
        var_y: var_y / batch_size,
        var_z: var_z / batch_size,
    }
}

fn classify(features: &Features) -> Classification {
    if features.var_x < 30.0 && features.var_y < 30.0 && features.var_z < 30.0 {
        Classification::LowActivity
    } else if features.var_x > 37.0 || features.var_y > 37.0 || features.var_z > 37.0 {
        Classification::HighActivity
    } else {
        Classification::Unknown
    }
}

#[tokio::main]
async fn main() {
    let batch_size = 100;
    let interval_seconds = 1;
    let mut counter = 0;

    println!("[INFO] Starting generation...");
    loop {
        counter += 1;
    
        let batch = generate_batch(batch_size);
        println!("[INFO] Generated batch {}", counter);

        let features = extract_features(&batch);
        println!("[INFO] Extracted features for batch {}", counter);

        let classification = classify(&features);
        println!("[INFO] Classification: {:?}", classification);
    
        println!("\n");

        sleep(Duration::from_secs(interval_seconds)).await;
    }
}
