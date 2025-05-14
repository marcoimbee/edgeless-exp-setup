use edgeless_function::*;
use serde::{Deserialize};
use serde_json;

struct HandleClassResultFun;

struct State {
    telemetry_id: i32,      // Identifies each telemetry sample couple
}

#[derive(Debug, Deserialize)]
enum Classification {
    Jogging,
    Walking,
    Standing,
    Stairs,
    Sitting,
}

#[derive(Debug, Deserialize)]
struct ClassificationPayload {
    batch_id: u64,
    classification: Classification,
}

static STATE: std::sync::OnceLock<std::sync::Mutex<State>> = std::sync::OnceLock::new();

impl EdgeFunction for HandleClassResultFun {

    // ------ EDGELESS FUNCTIONS REDEFINITION ------
    fn handle_cast(_src: InstanceId, encoded_message: &[u8]) {
        let mut state = STATE.get().unwrap().lock().unwrap();

        cast("metric", format!("function:begin:{}", state.telemetry_id).as_bytes());

        let display_class_result = |result: &Classification| -> String {
            match result {
                Classification::Jogging => "'JOGGING' activity detected".to_string(),
                Classification::Walking => "'WALKING' activity detected".to_string(),
                Classification::Standing => "'STANDING' activity detected".to_string(),
                Classification::Stairs => "'STAIRS' activity detected".to_string(),
                Classification::Sitting => "'SITTING' activity detected".to_string(),
            }
        };

        let str_message = core::str::from_utf8(encoded_message).unwrap();
        let class_result: ClassificationPayload = match serde_json::from_str(str_message) {
            Ok(parsed_class_result) => parsed_class_result,
            Err(err) => {
                log::info!("Failed to deserialize message: {}", err);
                return;
            }
        };

        log::info!("{}", display_class_result(&class_result.classification));

        let batch_id = class_result.batch_id;
        cast("aoi_measurement_end", format!("{}", batch_id).as_bytes());
    
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

edgeless_function::export!(HandleClassResultFun);
