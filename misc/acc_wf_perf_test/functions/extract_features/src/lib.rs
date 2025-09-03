use edgeless_function::*;
use serde::{Serialize, Deserialize};
use serde_json;
use statrs::statistics::{OrderStatistics, Statistics};

struct ExtractFeaturesFun;

struct State {
    telemetry_id: i32,            // Identifies each telemetry sample couple
}

#[derive(Debug, Deserialize)]
struct AccelerometerData {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Serialize)]
struct Features {
    mean_mag: f64,
    std_dev_mag: f64,
    min_mag: f64,
    max_mag: f64,
    coeff_var_mag: f64,
    percentile_25_mag: f64,
    percentile_75_mag: f64,

    mean_x: f64,
    std_dev_x: f64,
    min_x: f64,
    max_x: f64,
    coeff_var_x: f64,
    percentile_25_x: f64,
    percentile_75_x: f64,

    mean_y: f64,
    std_dev_y: f64,
    min_y: f64,
    max_y: f64,
    coeff_var_y: f64,
    percentile_25_y: f64,
    percentile_75_y: f64,

    mean_z: f64,
    std_dev_z: f64,
    min_z: f64,
    max_z: f64,
    coeff_var_z: f64,
    percentile_25_z: f64,
    percentile_75_z: f64,
}

#[derive(Debug, Deserialize)]
struct ReceivedPayload {
    batch_id: u64,
    batch: Vec<AccelerometerData>,
}

#[derive(Debug, Serialize)]
struct FeaturesPayload {
    batch_id: u64,
    features: Features,
}

static STATE: std::sync::OnceLock<std::sync::Mutex<State>> = std::sync::OnceLock::new();

impl EdgeFunction for ExtractFeaturesFun {

    // ------ EDGELESS FUNCTIONS REDEFINITION ------
    fn handle_cast(_src: InstanceId, encoded_message: &[u8]) {
        let mut state = STATE.get().unwrap().lock().unwrap();
        
        cast("metric", format!("function:begin:{}", state.telemetry_id).as_bytes());
        
        let str_message = core::str::from_utf8(encoded_message).unwrap();
        let received_data: ReceivedPayload = match serde_json::from_str(str_message) {
            Ok(parsed_received_data) => parsed_received_data,
            Err(err) => {
                log::info!("Failed to deserialize message: {}", err);
                return;
            }
        };

        let batch_id = received_data.batch_id;
        let accelerometer_data = received_data.batch;

        // --------- Feature extraction ---------
        // Calculate magnitudes for each sample
        let mut magnitudes: Vec<f64> = accelerometer_data
            .iter()
            .map(|data| {
                ((data.x.powi(2)) + (data.y.powi(2)) + (data.z.powi(2))).sqrt()
            })
            .collect();

        // Calculate statistics for Magnitudes
        let mean_mag = magnitudes.clone().mean();
        let std_dev_mag = magnitudes.clone().std_dev();
        let min_mag = magnitudes.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_mag = magnitudes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mut coeff_var_mag = 0.0;
        if mean_mag != 0.0 {
            coeff_var_mag = std_dev_mag / mean_mag;
        }
        let percentile_25_mag = magnitudes.percentile(25);
        let percentile_75_mag = magnitudes.percentile(75);
        
        // Extract x, y, z values for individual axis
        let mut x_vals: Vec<f64> = accelerometer_data.iter().map(|data| data.x).collect();
        let mut y_vals: Vec<f64> = accelerometer_data.iter().map(|data| data.y).collect();
        let mut z_vals: Vec<f64> = accelerometer_data.iter().map(|data| data.z).collect();

        // Function to compute statistics for an axis
        fn compute_stats(axis_data: &mut Vec<f64>) -> (f64, f64, f64, f64, f64, f64, f64) {
            let mean = axis_data.mean();
            let std_dev = axis_data.std_dev();
            let min = axis_data.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = axis_data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let mut coeff_var = 0.0;
            if mean != 0.0 {
                coeff_var = std_dev / mean;
            }
            let percentile_25 = axis_data.percentile(25);
            let percentile_75 = axis_data.percentile(75);

            (mean, std_dev, min, max, coeff_var, percentile_25, percentile_75)
        }
    
        // Compute statistics for X, Y, Z axes
        let (mean_x, std_dev_x, min_x, max_x, coeff_var_x, percentile_25_x, percentile_75_x) = compute_stats(&mut x_vals);
        let (mean_y, std_dev_y, min_y, max_y, coeff_var_y, percentile_25_y, percentile_75_y) = compute_stats(&mut y_vals);
        let (mean_z, std_dev_z, min_z, max_z, coeff_var_z, percentile_25_z, percentile_75_z) = compute_stats(&mut z_vals);
    
        // Store all the features
        let features = Features {
            mean_mag,
            std_dev_mag,
            min_mag,
            max_mag,
            coeff_var_mag,
            percentile_25_mag,
            percentile_75_mag,

            mean_x,
            std_dev_x,
            min_x,
            max_x,
            coeff_var_x,
            percentile_25_x,
            percentile_75_x,
    
            mean_y,
            std_dev_y,
            min_y,
            max_y,
            coeff_var_y,
            percentile_25_y,
            percentile_75_y,
    
            mean_z,
            std_dev_z,
            min_z,
            max_z,
            coeff_var_z,
            percentile_25_z,
            percentile_75_z,
        };

        log::info!("Features have been extracted");

        let payload = FeaturesPayload {
            batch_id,
            features,
        };

        let serialized_features = match serde_json::to_string(&payload) {
            Ok(json) => json,
            Err(e) => {
                log::info!("Error serializing extracted features: {}", e);
                String::new()
            }
        };

        cast("extracted_features", serialized_features.as_bytes());
        
        cast("metric", format!("function:end:{}", state.telemetry_id).as_bytes());
        state.telemetry_id += 1;
    }

    fn handle_call(_src: InstanceId, _encoded_message: &[u8]) -> CallRet {
        log::info!("handle_call() called");
        CallRet::NoReply
    }

    fn handle_init(_payload: Option<&[u8]>, _init_state: Option<&[u8]>) {
        edgeless_function::init_logger();
        log::info!("started");

        let _ = STATE.set(std::sync::Mutex::new(
            State { 
                telemetry_id: 0,
            }
        ));
    }

    fn handle_stop() {
        log::info!("stopped");
    }
}

edgeless_function::export!(ExtractFeaturesFun);