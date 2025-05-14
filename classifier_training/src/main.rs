use std::collections::HashMap;
use std::error::Error;
use csv::Reader;
use statrs::statistics::{OrderStatistics, Statistics};
use std::f64;
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::ensemble::random_forest_classifier::RandomForestClassifier;
use std::time::Instant;
use serde::{Serialize, Deserialize};
use bincode;
use rusqlite::{params, Connection, Result};

#[derive(Serialize, Deserialize)]
struct ModelMetadata {
    classifier_base64: String,              // Serialized model as Base64 (OK for JSON in SQLite table)
}

#[derive(Serialize, Deserialize)]
struct SerializableModel {
    classifier: RandomForestClassifier<f64, i32, DenseMatrix<f64>, Vec<i32>>,
}

// Structure to hold a single row of data
#[derive(Debug, Deserialize, Clone)]
struct DataRow {
    activity: String,
    accel_x: f64,
    accel_y: f64,
    accel_z: f64,
}

// Structure for extracted features
#[derive(Debug, Clone)]
struct Features([f64; 28]);

// Read CSV file and load data into a vector
fn read_csv(file_path: &str) -> Result<Vec<DataRow>, Box<dyn Error>> {
    let mut rdr = Reader::from_path(file_path)?;
    let mut data = Vec::new();

    for result in rdr.deserialize() {
        let record: DataRow = result?;
        data.push(record);
    }

    Ok(data)
}

fn print_min_max_per_activity(data: Vec<DataRow>) {
    let mut stats: HashMap<String, (f64, f64, f64, f64, f64, f64)> = HashMap::new();

    for point in data {
        stats
            .entry(point.activity.clone())
            .and_modify(|(min_x, min_y, min_z, max_x, max_y, max_z)| {
                *min_x = (*min_x).min(point.accel_x);
                *min_y = (*min_y).min(point.accel_y);
                *min_z = (*min_z).min(point.accel_z);
                *max_x = (*max_x).max(point.accel_x);
                *max_y = (*max_y).max(point.accel_y);
                *max_z = (*max_z).max(point.accel_z);
            })
            .or_insert((point.accel_x, point.accel_y, point.accel_z, point.accel_x, point.accel_y, point.accel_z));
    }

    for (activity, (min_x, min_y, min_z, max_x, max_y, max_z)) in stats {
        println!("Activity: {}", activity);
        println!("  Min X: {:.2}, Min Y: {:.2}, Min Z: {:.2}", min_x, min_y, min_z);
        println!("  Max X: {:.2}, Max Y: {:.2}, Max Z: {:.2}", max_x, max_y, max_z);
    }
}


fn compute_stats(mut data: Vec<f64>) -> (f64, f64, f64, f64, f64, f64, f64) {
    if data.is_empty() {
        return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }

    let mean = data.clone().mean();
    let std_dev = data.clone().std_dev();
    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut coeff_var = 0.0;
    if mean != 0.0 {
        coeff_var = std_dev / mean;
    }

    let p25 = data.percentile(25);
    let p75 = data.percentile(75);

    (mean, std_dev, min, max, coeff_var, p25, p75)
}

// Compute statistical features for a chunk
fn compute_features(chunk: &[DataRow]) -> Features {
    let mut x_vals = Vec::with_capacity(chunk.len());
    let mut y_vals = Vec::with_capacity(chunk.len());
    let mut z_vals = Vec::with_capacity(chunk.len());
    let mut magnitudes = Vec::with_capacity(chunk.len());

    for row in chunk {
        let mag = (row.accel_x.powi(2) + row.accel_y.powi(2) + row.accel_z.powi(2)).sqrt();
        x_vals.push(row.accel_x);
        y_vals.push(row.accel_y);
        z_vals.push(row.accel_z);
        magnitudes.push(mag);
    }

    let (mean_mag, std_dev_mag, min_mag, max_mag, coeff_var_mag, p25_mag, p75_mag) = compute_stats(magnitudes);
    let (mean_x, std_dev_x, min_x, max_x, coeff_var_x, p25_x, p75_x) = compute_stats(x_vals);
    let (mean_y, std_dev_y, min_y, max_y, coeff_var_y, p25_y, p75_y) = compute_stats(y_vals);
    let (mean_z, std_dev_z, min_z, max_z, coeff_var_z, p25_z, p75_z) = compute_stats(z_vals);

    Features([
        mean_mag, std_dev_mag, min_mag, max_mag, coeff_var_mag, p25_mag, p75_mag,
        mean_x, std_dev_x, min_x, max_x, coeff_var_x, p25_x, p75_x,
        mean_y, std_dev_y, min_y, max_y, coeff_var_y, p25_y, p75_y,
        mean_z, std_dev_z, min_z, max_z, coeff_var_z, p25_z, p75_z,
    ])
}

fn extract_features(data: Vec<DataRow>) -> HashMap<String, Vec<Features>> {
    let mut activity_map: HashMap<String, Vec<DataRow>> = HashMap::new();

    // Group by activity
    for row in data {
        activity_map.entry(row.activity.clone()).or_insert_with(Vec::new).push(row);
    }

    // Convert each activity's data into feature vectors
    let mut feature_map: HashMap<String, Vec<Features>> = HashMap::new();
    
    for (activity, rows) in &activity_map {
        let chunks: Vec<&[DataRow]> = rows.chunks(100).collect();
        
        // Compute features for each chunk
        let feature_vectors: Vec<Features> = chunks.iter().map(|chunk| compute_features(chunk)).collect();
        feature_map.insert(activity.clone(), feature_vectors.clone());

        println!("[INFO] Processed '{}' activity data, #feature vectors: {}", activity, feature_vectors.len());
    }

    feature_map
}


fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "preprocessed_dataset.csv";

    println!("[INFO] Extracting features...");
    let data = read_csv(input_file)?;
    let feature_map = extract_features(data.clone());

    print_min_max_per_activity(data.clone());

    println!("----------------------------------------------------------------");

    let mut features_data: Vec<Vec<f64>> = Vec::new();
    let mut labels: Vec<i32> = Vec::new();

    for (activity, features_vectors) in feature_map {
        let label = match activity.as_str() {
            "jogging" => 0,
            "walking" => 1,
            "standing" => 2,
            "stairs" => 3,
            "sitting" => 4,
            _ => -1,
        };

        for feature_vector in features_vectors {
            features_data.push(feature_vector.0.to_vec());
            labels.push(label);
        }
    }

    let x = DenseMatrix::from_2d_vec(&features_data);

    // Train the classifier
    println!("[INFO] Training...");
    let start = Instant::now();
    let classifier = RandomForestClassifier::fit(&x, &labels, Default::default()).unwrap();
    let duration = start.elapsed();
    println!("[INFO] Random Forest classifier trained in {:.2?}", duration);


    println!("----------------------------------------------------------------");
    println!("[INFO] Testing:");
    let data = vec![
        vec![6.05, 2.15, 5.08, 90.005, 67.018, 12.009, 78.620, 56.08, 12.18, 45.09, 45.007, 45.017, 78.009, 0.620, 0.07, 11.16, 14.08, 33.006,44.016, 13.008, 0.620, 0.07, 783.15, 13.08, 0.006, 84.015, 13.008, 90.620],  // Low activity  
        vec![0.2, 0.35, 0.22, 0.03, 0.045, 0.025, 0.650, 0.22, 0.33, 0.21, 0.028, 0.043, 0.024, 0.650, 0.21, 0.31, 0.2, 0.026, 0.041, 0.022, 0.650, 0.2, 0.3, 0.19, 0.025, 0.04, 0.021, 0.650],  // Moderate activity  
        vec![0.4, 0.6, 0.45, 0.07, 0.09, 0.05, 0.670, 0.42, 0.58, 0.44, 0.065, 0.088, 0.048, 0.670, 0.41, 0.55, 0.42, 0.063, 0.085, 0.047, 0.670, 0.4, 0.53, 0.41, 0.06, 0.08, 0.045, 0.670],  // High activity  
    ];

    for (i, test_sample_vec) in data.iter().enumerate() {
        let test_sample = DenseMatrix::from_2d_vec(&vec![test_sample_vec.clone()]);
        let prediction = classifier.predict(&test_sample).unwrap();
        println!("Test Sample {}: Predicted Class: {:?}", i + 1, prediction);
    }

    println!("----------------------------------------------------------------");
    println!("[INFO] Serializing...");

    let serializable_model = SerializableModel { classifier };
    let serialized = bincode::serialize(&serializable_model).unwrap();

    let encoded_model = base64::encode(&serialized);

    let metadata = ModelMetadata {
        classifier_base64: encoded_model,
    };

    let metadata_json = serde_json::to_string(&metadata).unwrap();
    println!("[INFO] The model has been serialized");

    println!("----------------------------------------------------------------");
    println!("[INFO] Saving into SQLite DB...");

    let sqlite_conn = Connection::open("<path-to-SQLite-EDGELESS-db>")?;
    let _ = sqlite_conn.execute(
        "INSERT INTO WorkflowState (id, metadata) VALUES (?1, ?2)",
        params!["base64_model", metadata_json],
    );

    println!("[INFO] Model saved to SQLite DB");

    Ok(())
}