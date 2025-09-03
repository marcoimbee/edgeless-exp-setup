use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::ensemble::random_forest_classifier::RandomForestClassifier;
use std::fs::File;
use std::io::{Write, Read};
use bincode;
use serde::{Serialize, Deserialize};
use std::fs;
use rusqlite::{params, Connection, Result};


#[derive(Serialize, Deserialize)]
struct ModelMetadata {
    classifier_base64: String,              // Serialized model as Base64 (OK for JSON in SQLite table)
}

#[derive(Serialize, Deserialize)]
struct SerializableModel {
    classifier: RandomForestClassifier<f64, i32, DenseMatrix<f64>, Vec<i32>>,
}

fn main() -> Result<()> {
    let conn = Connection::open("/home/marco/Desktop/edgeless_db.db")?;
    println!("Connected to EDGELESS DB.");

    // let data = vec![
    //     vec![0.05, 0.15, 0.08, 0.005, 0.018, 0.009, 0.620, 0.08, 0.18, 0.09, 0.007, 0.017, 0.009, 0.620, 0.07, 0.16, 0.08, 0.006, 0.016, 0.008, 0.620, 0.07, 0.15, 0.08, 0.006, 0.015, 0.008, 0.620],  // Low activity  
    //     vec![0.2, 0.35, 0.22, 0.03, 0.045, 0.025, 0.650, 0.22, 0.33, 0.21, 0.028, 0.043, 0.024, 0.650, 0.21, 0.31, 0.2, 0.026, 0.041, 0.022, 0.650, 0.2, 0.3, 0.19, 0.025, 0.04, 0.021, 0.650],  // Moderate activity  
    //     vec![0.4, 0.6, 0.45, 0.07, 0.09, 0.05, 0.670, 0.42, 0.58, 0.44, 0.065, 0.088, 0.048, 0.670, 0.41, 0.55, 0.42, 0.063, 0.085, 0.047, 0.670, 0.4, 0.53, 0.41, 0.06, 0.08, 0.045, 0.670],  // High activity  
    // ];

    // let labels = vec![0, 1, 2]; // Different classes for different activity levels

    // let x = DenseMatrix::from_2d_vec(&data);
    // let y = labels.iter().map(|&x| x as i32).collect::<Vec<i32>>(); // Fix: Convert labels to i32

    // // Train the classifier
    // let classifier = RandomForestClassifier::fit(&x, &y, Default::default()).unwrap();

    // for (i, test_sample_vec) in data.iter().enumerate() {
    //     let test_sample = DenseMatrix::from_2d_vec(&vec![test_sample_vec.clone()]);
    //     let prediction = classifier.predict(&test_sample).unwrap();
    //     println!("Test Sample {}: Predicted Class: {:?}", i + 1, prediction);
    // }


    // // Serialization
    // let serializable_model = SerializableModel { classifier };

    // let serialized = bincode::serialize(&serializable_model).unwrap();

    // let encoded_model = base64::encode(&serialized);

    // let metadata = ModelMetadata {
    //     classifier_base64: encoded_model,
    // };

    // let metadata_json = serde_json::to_string(&metadata).unwrap();

    // conn.execute(
    //     "INSERT INTO WorkflowState (id, metadata) VALUES (?1, ?2)",
    //     params!["base64_model", metadata_json],
    // );

    // println!("Saved to SQLite.");

    // println!("------------------------------");

    let mut stmt = conn.prepare("SELECT metadata FROM WorkflowState WHERE id = ?1")?;
    let mut rows = stmt.query(params!["base64_model"])?;

    if let Some(row) = rows.next()? {
        let metadata_json: String = row.get(0)?;
        let metadata: ModelMetadata = serde_json::from_str(&metadata_json).unwrap();

        let serialized_model = base64::decode(&metadata.classifier_base64).unwrap();

        let deserialized_model: SerializableModel = bincode::deserialize(&serialized_model).unwrap();

        println!("Deserialized.");

        let test_samples = vec![
            vec![0.06, 0.14, 0.09, 0.004, 0.017, 0.008, 0.618, 0.09, 0.17, 0.085, 0.006, 0.016, 0.008, 0.618, 0.08, 0.16, 0.08, 0.005, 0.015, 0.007, 0.618, 0.08, 0.14, 0.075, 0.004, 0.014, 0.007, 0.618], // Low activity
            vec![0.18, 0.32, 0.2, 0.025, 0.04, 0.02, 0.645, 0.2, 0.31, 0.19, 0.024, 0.038, 0.019, 0.645, 0.19, 0.3, 0.185, 0.023, 0.037, 0.018, 0.645, 0.18, 0.29, 0.18, 0.022, 0.036, 0.017, 0.645], // Moderate activity
            vec![0.38, 0.58, 0.42, 0.06, 0.085, 0.045, 0.665, 0.4, 0.55, 0.4, 0.058, 0.082, 0.043, 0.665, 0.39, 0.53, 0.395, 0.056, 0.08, 0.042, 0.665, 0.38, 0.51, 0.39, 0.054, 0.078, 0.041, 0.665], // High activity
        ];

        for (i, test_sample_vec) in test_samples.iter().enumerate() {
            let test_sample = DenseMatrix::from_2d_vec(&vec![test_sample_vec.clone()]);
            let prediction = deserialized_model.classifier.predict(&test_sample).unwrap();
            println!("Test Sample {}: Predicted Class: {:?}", i + 1, prediction);
        }
    }

    // match bincode::serialize(&serializable_model) {
    //     Ok(serialized) => {
    //         fs::write("random_forest.bin", &serialized).unwrap();
    //         println!("Serialization successful!");

    //         // Insert the serialized classifier
    //         if let Err(e) = conn.execute(
    //             "INSERT INTO edgeless_data (serialized_classifier) VALUES (?1)",
    //             params![serialized],
    //         ) {
    //             eprintln!("Failed to insert data into SQLite: {}", e);
    //             return Err(e);
    //         }
    //     },
    //     Err(e) => {
    //         eprintln!("Serialization failed: {:?}", e);
    //         return;
    //     }
    // }

    // Deserialization
    // let serialized = fs::read("random_forest.bin").unwrap();
    // match bincode::deserialize::<SerializableModel>(&serialized) {
    //     Ok(deserialized_model) => {
    //         println!("Deserialization successful!");

    //         // Extract the classifier from the deserialized model
    //         let deserialized_classifier = deserialized_model.classifier;

    //         // Now you can use the deserialized classifier to make predictions
    //         let test_samples = vec![
    //             vec![0.06, 0.14, 0.09, 0.004, 0.017, 0.008, 0.618, 0.09, 0.17, 0.085, 0.006, 0.016, 0.008, 0.618, 0.08, 0.16, 0.08, 0.005, 0.015, 0.007, 0.618, 0.08, 0.14, 0.075, 0.004, 0.014, 0.007, 0.618], // Low activity
    //             vec![0.18, 0.32, 0.2, 0.025, 0.04, 0.02, 0.645, 0.2, 0.31, 0.19, 0.024, 0.038, 0.019, 0.645, 0.19, 0.3, 0.185, 0.023, 0.037, 0.018, 0.645, 0.18, 0.29, 0.18, 0.022, 0.036, 0.017, 0.645], // Moderate activity
    //             vec![0.38, 0.58, 0.42, 0.06, 0.085, 0.045, 0.665, 0.4, 0.55, 0.4, 0.058, 0.082, 0.043, 0.665, 0.39, 0.53, 0.395, 0.056, 0.08, 0.042, 0.665, 0.38, 0.51, 0.39, 0.054, 0.078, 0.041, 0.665], // High activity
    //         ];

    //         for (i, test_sample_vec) in test_samples.iter().enumerate() {
    //             let test_sample = DenseMatrix::from_2d_vec(&vec![test_sample_vec.clone()]);
    //             let prediction = deserialized_classifier.predict(&test_sample).unwrap();
    //             println!("Test Sample {}: Predicted Class: {:?}", i + 1, prediction);
    //         }
    //     },
    //     Err(e) => {
    //         eprintln!("Deserialization failed: {:?}", e);
    //     }
    // }

    Ok(())
}
