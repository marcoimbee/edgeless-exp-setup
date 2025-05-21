use edgeless_function::*;
use serde::{Deserialize};
use serde_json;

struct HandleClassResultFun;

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

impl EdgeFunction for HandleClassResultFun {

    // ------ EDGELESS FUNCTIONS REDEFINITION ------
    fn handle_cast(_src: InstanceId, encoded_message: &[u8]) {
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
        cast("ttc_measurement_end", format!("{}", batch_id).as_bytes());
    }

    fn handle_call(_src: InstanceId, _encoded_message: &[u8]) -> CallRet {
        log::info!("handle_call() called");
        CallRet::NoReply
    }

 fn handle_init(_payload: Option<&[u8]>, _init_state: Option<&[u8]>) {
        edgeless_function::init_logger();
        log::info!("Started");
    }

    fn handle_stop() {
        log::info!("Stopped");
    }
}

edgeless_function::export!(HandleClassResultFun);
