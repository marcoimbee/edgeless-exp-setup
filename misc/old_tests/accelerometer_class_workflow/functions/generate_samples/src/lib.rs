use edgeless_function::*;
use serde::Serialize;
use serde_json;
struct GenerateSamplesFun;

impl EdgeFunction for GenerateSamplesFun {

    // ------ EDGELESS FUNCTIONS REDEFINITION ------

    // Starts and generates 100 samples every 5 seconds. 
    // Sends them to the next function in the workflow
    fn handle_cast(_src: InstanceId, _encoded_message: &[u8]) {
        log::info!("handle_cast() called");

        let batch_size: usize = 100;
        let interval_ms: u64 = 5000;            // generate values every 5 seconds

        #[derive(Debug, Serialize)]
        struct AccelerometerData {
            x: f64,
            y: f64,
            z: f64,
        }
    
        let mut batch = Vec::with_capacity(batch_size);

        for _ in 0..batch_size {                // Generating batch of accelerometric data
            batch.push(
                AccelerometerData {         // TODO: find a way to generate randvalues here
                    x: 12.67,
                    y: 7.23,
                    z: 56.1,
                }
            );
        }

        log::info!("Generated a new batch");

        let serialized_batch = match serde_json::to_string(&batch) {
            Ok(json) => json,
            Err(e) => {
                log::info!("Error serializing accelerometric data batch: {}", e);
                String::new()
            }
        };

        cast("generated_samples", serialized_batch.as_bytes());

        delayed_cast(interval_ms, "self", b"");
    }

    fn handle_call(_src: InstanceId, _encoded_message: &[u8]) -> CallRet {
        log::info!("handle_call() called");
        CallRet::NoReply
    }

    fn handle_init(_payload: Option<&[u8]>, _init_state: Option<&[u8]>) {
        edgeless_function::init_logger();
        log::info!("started");
        cast("self", b"");          // Action happens in handle_cast()
    }

    fn handle_stop() {
        log::info!("stopped");
    }
}

edgeless_function::export!(GenerateSamplesFun);