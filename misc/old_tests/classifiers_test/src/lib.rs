use edgeless_function::*;
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::ensemble::random_forest_classifier::RandomForestClassifier;
use std::sync::{Mutex, OnceLock};
use nalgebra::{Matrix3, Vector3, DMatrix};
use ndarray::Array1;
use statrs::statistics::{Mean, Variance, OrderStatistics, Skewness};
use ndarray::Data;

struct TestClassificationFun;

struct State {
    classifier: RandomForestClassifier<f64, i32, DenseMatrix<f64>, Vec<i32>>,
}

// Static variable to hold the classifier state
static STATE: OnceLock<Mutex<State>> = OnceLock::new();

impl EdgeFunction for TestClassificationFun {
    fn handle_init(_payload: Option<&[u8]>, _init_state: Option<&[u8]>) {
        edgeless_function::init_logger();
        log::info!("Started");

        let data = vec![
            vec![0.05, 0.15, 0.08, 0.005, 0.018, 0.009, 0.620, 0.08, 0.18, 0.09, 0.007, 0.017, 0.009, 0.620, 0.07, 0.16, 0.08, 0.006, 0.016, 0.008, 0.620, 0.07, 0.15, 0.08, 0.006, 0.015, 0.008, 0.620],  // Low activity  
            vec![0.2, 0.35, 0.22, 0.03, 0.045, 0.025, 0.650, 0.22, 0.33, 0.21, 0.028, 0.043, 0.024, 0.650, 0.21, 0.31, 0.2, 0.026, 0.041, 0.022, 0.650, 0.2, 0.3, 0.19, 0.025, 0.04, 0.021, 0.650],  // Moderate activity  
            vec![0.4, 0.6, 0.45, 0.07, 0.09, 0.05, 0.670, 0.42, 0.58, 0.44, 0.065, 0.088, 0.048, 0.670, 0.41, 0.55, 0.42, 0.063, 0.085, 0.047, 0.670, 0.4, 0.53, 0.41, 0.06, 0.08, 0.045, 0.670],  // High activity  
        ];

        let labels = vec![0, 1, 2]; // Different classes for different activity levels

        let x = DenseMatrix::from_2d_vec(&data);
        let y = labels.iter().map(|&x| x as i32).collect::<Vec<i32>>(); // Fix: Convert labels to i32

        // Train the classifier
        let classifier = RandomForestClassifier::fit(&x, &y, Default::default()).unwrap();

        // Store the trained classifier in the state
        let _ = STATE.set(Mutex::new(State { classifier }));
            // .expect("Failed to set state");

        // Schedule handle_cast() to run after 2000ms
        delayed_cast(2000, "self", b"");
    }

    fn handle_cast(_src: InstanceId, _encoded_message: &[u8]) {
        // Ensure the classifier state is available
        let state = STATE.get().expect("Classifier not initialized").lock().unwrap();

        // // Test sample for classification
        // let test_sample_vec = vec![0.1, 0.2, 0.1, 0.01, 0.02, 0.01, 0.635, 0.1, 0.2, 0.1, 0.01, 0.02, 0.01, 0.635, 0.1, 0.2, 0.1, 0.01, 0.02, 0.01, 0.635, 0.1, 0.2, 0.1, 0.01, 0.02, 0.01, 0.635];
        // // let test_sample = DenseMatrix::from_2d_vec(&test_sample_vec);
        // let test_sample = DenseMatrix::from_2d_vec(&vec![test_sample_vec]);

        // // Use stored classifier to make prediction
        // let prediction = state.classifier.predict(&test_sample).unwrap();

        // log::info!("Predicted Class: {:?}", prediction);

        let test_samples = vec![
            vec![0.06, 0.14, 0.09, 0.004, 0.017, 0.008, 0.618, 0.09, 0.17, 0.085, 0.006, 0.016, 0.008, 0.618, 0.08, 0.16, 0.08, 0.005, 0.015, 0.007, 0.618, 0.08, 0.14, 0.075, 0.004, 0.014, 0.007, 0.618], // Low activity (Class 0)
            vec![0.18, 0.32, 0.2, 0.025, 0.04, 0.02, 0.645, 0.2, 0.31, 0.19, 0.024, 0.038, 0.019, 0.645, 0.19, 0.3, 0.185, 0.023, 0.037, 0.018, 0.645, 0.18, 0.29, 0.18, 0.022, 0.036, 0.017, 0.645], // Moderate activity (Class 1)
            vec![0.38, 0.58, 0.42, 0.06, 0.085, 0.045, 0.665, 0.4, 0.55, 0.4, 0.058, 0.082, 0.043, 0.665, 0.39, 0.53, 0.395, 0.056, 0.08, 0.042, 0.665, 0.38, 0.51, 0.39, 0.054, 0.078, 0.041, 0.665], // High activity (Class 2)
            vec![0.5, 0.7, 0.55, 0.1, 0.12, 0.07, 0.68, 0.52, 0.69, 0.54, 0.098, 0.118, 0.068, 0.68, 0.51, 0.67, 0.53, 0.096, 0.115, 0.066, 0.68, 0.5, 0.65, 0.52, 0.094, 0.112, 0.064, 0.68], // Very high activity (Possible Class 3)
            vec![0.02, 0.05, 0.03, 0.002, 0.006, 0.003, 0.61, 0.03, 0.04, 0.025, 0.001, 0.005, 0.002, 0.61, 0.025, 0.035, 0.02, 0.0015, 0.0045, 0.0025, 0.61, 0.02, 0.03, 0.018, 0.001, 0.004, 0.002, 0.61], // Almost no activity (Possible Class 4)
        ];

        // Convert samples into DenseMatrix for testi
        for (i, test_sample_vec) in test_samples.iter().enumerate() {
            let test_sample = DenseMatrix::from_2d_vec(&vec![test_sample_vec.clone()]);
            let prediction = state.classifier.predict(&test_sample).unwrap();
            log::info!("Test Sample {}: Predicted Class: {:?}", i + 1, prediction);
        }
    }

    fn handle_call(_src: InstanceId, _encoded_message: &[u8]) -> CallRet {
        log::info!("handle_call() called");
        CallRet::NoReply
    }

    fn handle_stop() {
        log::info!("Stopped");
    }
}

edgeless_function::export!(TestClassificationFun);
