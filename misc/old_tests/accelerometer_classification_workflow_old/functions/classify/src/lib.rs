use edgeless_function::*;
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::ensemble::random_forest_classifier::RandomForestClassifier;
use std::sync::{Mutex, OnceLock};
use serde::{Serialize, Deserialize};

struct ClassifyFun;

struct State {
    classifier: RandomForestClassifier<f64, i32, DenseMatrix<f64>, Vec<i32>>,
}

static STATE: OnceLock<Mutex<State>> = OnceLock::new();

impl EdgeFunction for ClassifyFun {

    // ------ EDGELESS FUNCTIONS REDEFINITION ------
    fn handle_cast(_src: InstanceId, encoded_message: &[u8]) {
        
        let state = STATE.get().expect("Classifier not initialized").lock().unwrap();
        
        #[derive(Debug, Deserialize)]
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
            features: Features,
        }

        #[derive(Debug, Serialize)]
        enum Classification {
            LowActivity,
            ModerateActivity,
            HighActivity,
        }

        #[derive(Debug, Serialize)]
        struct ClassificationPayload {
            batch_id: u64,
            classification: Classification,
        }

        let str_message = core::str::from_utf8(encoded_message).unwrap();
        let received_data: ReceivedPayload = match serde_json::from_str(str_message) {
            Ok(parsed_received_data) => parsed_received_data,
            Err(err) => {
                log::info!("Failed to deserialize message: {}", err);
                return;
            }
        };

        let batch_id = received_data.batch_id;
        let extracted_features = received_data.features;

        // Features struct into Vec<f64> array
        let features_vec: Vec<f64> = vec![
            extracted_features.mean_mag,
            extracted_features.std_dev_mag,
            extracted_features.min_mag,
            extracted_features.max_mag,
            extracted_features.coeff_var_mag,
            extracted_features.percentile_25_mag,
            extracted_features.percentile_75_mag,

            extracted_features.mean_x,
            extracted_features.std_dev_x,
            extracted_features.min_x,
            extracted_features.max_x,
            extracted_features.coeff_var_x,
            extracted_features.percentile_25_x,
            extracted_features.percentile_75_x,

            extracted_features.mean_y,
            extracted_features.std_dev_y,
            extracted_features.min_y,
            extracted_features.max_y,
            extracted_features.coeff_var_y,
            extracted_features.percentile_25_y,
            extracted_features.percentile_75_y,

            extracted_features.mean_z,
            extracted_features.std_dev_z,
            extracted_features.min_z,
            extracted_features.max_z,
            extracted_features.coeff_var_z,
            extracted_features.percentile_25_z,
            extracted_features.percentile_75_z,
        ];

        let sample = DenseMatrix::from_2d_vec(&vec![features_vec]);

        let prediction = state.classifier.predict(&sample).unwrap();

        let classification_result;
        if prediction[0] == 0 {
            classification_result = Classification::LowActivity;
        } else if prediction[0] == 1 {
            classification_result = Classification::ModerateActivity;
        } else {
            classification_result = Classification::HighActivity;
        }

        log::info!("Classified the received features");

        let payload = ClassificationPayload {
            batch_id,
            classification: classification_result,
        };

        let serialized_classification_result = match serde_json::to_string(&payload) {
            Ok(json) => json,
            Err(e) => {
                log::info!("Error serializing classification result: {}", e);
                String::new()
            }
        };

        cast("classification_result", serialized_classification_result.as_bytes());
    }

    fn handle_call(_src: InstanceId, _encoded_message: &[u8]) -> CallRet {
        log::info!("handle_call() called");
        CallRet::NoReply
    }

 fn handle_init(_payload: Option<&[u8]>, _init_state: Option<&[u8]>) {
        edgeless_function::init_logger();

        // ----- TEMPORARY TRAINING STEP HERE ------
        let data = vec![
            vec![0.05, 0.15, 0.08, 0.005, 0.018, 0.009, 0.620, 0.08, 0.18, 0.09, 0.007, 0.017, 0.009, 0.620, 0.07, 0.16, 0.08, 0.006, 0.016, 0.008, 0.620, 0.07, 0.15, 0.08, 0.006, 0.015, 0.008, 0.620],  // Low activity  
            vec![0.2, 0.35, 0.22, 0.03, 0.045, 0.025, 0.650, 0.22, 0.33, 0.21, 0.028, 0.043, 0.024, 0.650, 0.21, 0.31, 0.2, 0.026, 0.041, 0.022, 0.650, 0.2, 0.3, 0.19, 0.025, 0.04, 0.021, 0.650],  // Moderate activity  
            vec![0.4, 0.6, 0.45, 0.07, 0.09, 0.05, 0.670, 0.42, 0.58, 0.44, 0.065, 0.088, 0.048, 0.670, 0.41, 0.55, 0.42, 0.063, 0.085, 0.047, 0.670, 0.4, 0.53, 0.41, 0.06, 0.08, 0.045, 0.670],  // High activity  
        ];

        let labels = vec![0, 1, 2]; // Classes

        let x = DenseMatrix::from_2d_vec(&data);
        let y = labels.iter().map(|&x| x as i32).collect::<Vec<i32>>();

        // Train the classifier
        let classifier = RandomForestClassifier::fit(&x, &y, Default::default()).unwrap();

        // Store the trained classifier in the state
        let _ = STATE.set(Mutex::new(State { classifier }));

        log::info!("Started, trained Random Forest classifier");
    }

    fn handle_stop() {
        log::info!("stopped");
    }
}

edgeless_function::export!(ClassifyFun);
