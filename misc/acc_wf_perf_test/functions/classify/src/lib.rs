use edgeless_function::*;
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::ensemble::random_forest_classifier::RandomForestClassifier;
use std::sync::{Mutex, OnceLock};
use serde::{Serialize, Deserialize};
use bincode;

struct ClassifyFun;

struct State {
    telemetry_id: i32,            // Identifies each telemetry sample couple
    classifier: RandomForestClassifier<f64, i32, DenseMatrix<f64>, Vec<i32>>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SqlxClassifierData {
    id: String,
    metadata: ClassifierData,
}

// Metadata to hold the classifier as base64 string
#[derive(serde::Serialize, serde::Deserialize, Default, Debug)]
struct ClassifierData {
    classifier_base64: String, // Serialized model as base64
}

// This will hold the actual classifier after decoding and deserializing
#[derive(serde::Serialize, serde::Deserialize)]
struct SerializableModel {
    classifier: RandomForestClassifier<f64, i32, DenseMatrix<f64>, Vec<i32>>,
}

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
    Jogging,
    Walking,
    Standing,
    Stairs,
    Sitting,
}

#[derive(Debug, Serialize)]
struct ClassificationPayload {
    batch_id: u64,
    classification: Classification,
}

static STATE: OnceLock<Mutex<State>> = OnceLock::new();

fn call_wrapper(msg: &str) -> Option<SqlxClassifierData> {
    match call("sqlx_database", msg.as_bytes()) {
        CallRet::Reply(msg) => {
            let reply = std::str::from_utf8(&msg).unwrap_or("not UTF-8");
            log::info!("Got reply from DB");
            let cur_state: SqlxClassifierData = serde_json::from_str(reply).unwrap_or_default();
            Some(cur_state)
        }
        CallRet::NoReply => {
            log::warn!("Received empty reply from the DB");
            None
        }
        CallRet::Err => {
            log::error!("Error while calling the DB");
            None
        }
    }
}

impl EdgeFunction for ClassifyFun {

    // ------ EDGELESS FUNCTIONS REDEFINITION ------
    fn handle_cast(_src: InstanceId, encoded_message: &[u8]) {
        let mut state = STATE.get().expect("Classifier not initialized").lock().unwrap();

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
            classification_result = Classification::Jogging;
        } else if prediction[0] == 1 {
            classification_result = Classification::Walking;
        } else if prediction[0] == 2 {
            classification_result = Classification::Standing;
        } else if prediction[0] == 3 {
            classification_result = Classification::Stairs;
        } else {
            classification_result = Classification::Sitting;
        }

        log::info!("Classified the received features: {:?}", prediction);

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
    
        cast("metric", format!("function:end:{}", state.telemetry_id).as_bytes());
        state.telemetry_id += 1;
    }

    fn handle_call(_src: InstanceId, _encoded_message: &[u8]) -> CallRet {
        log::info!("handle_call() called");
        CallRet::NoReply
    }

 fn handle_init(_payload: Option<&[u8]>, _init_state: Option<&[u8]>) {
        edgeless_function::init_logger();

        if let Some(result) = call_wrapper("SELECT id, metadata FROM WorkflowState LIMIT 1",) {
            let classifier_base64 = result.metadata.classifier_base64;

            let serialized_model = base64::decode(&classifier_base64).unwrap();

            let deserialized_model: SerializableModel = bincode::deserialize(&serialized_model).unwrap();

            let _ = STATE.set(Mutex::new(
                State { 
                    telemetry_id: 0,
                    classifier: deserialized_model.classifier 
                }
            ));
        }

        log::info!("Started, retrieved Random Forest classifier, saved it in the function's state");
    }

    fn handle_stop() {
        log::info!("stopped");
    }
}

edgeless_function::export!(ClassifyFun);
