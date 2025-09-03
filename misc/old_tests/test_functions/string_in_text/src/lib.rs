use edgeless_function::*;
use serde_json::Value;

struct StringInTextFun;

impl EdgeFunction for StringInTextFun {
    fn handle_cast(_src: InstanceId, encoded_message: &[u8]) {
        let str_message = core::str::from_utf8(encoded_message).unwrap();

        log::info!("string_in_text: called with '{}'", str_message);

        if let Ok(json) = serde_json::from_str::<Value>(str_message) {
            if let (Some(text), Some(word)) = (
                json.get("Text").and_then(|v| v.as_str()),
                json.get("Word").and_then(|v| v.as_str()),
            ) {
                let result = text.contains(word);
                log::info!("Checking if '{}' contains '{}': {}", text, word, result);

                cast("result", format!("{}", result).as_bytes());
            } else {
                log::error!("JSON does not contain valid 'Text' and 'Word' fields");
            }
        } else {
            log::error!("failed to parse JSON from message: {}", str_message)
        }
    }

    fn handle_call(_src: InstanceId, _encoded_message: &[u8]) -> CallRet {
        CallRet::NoReply
    }

    fn handle_init(_payload: Option<&[u8]>, _init_state: Option<&[u8]>) {
        edgeless_function::init_logger();
        log::info!("string_in_text: started");
    }

    fn handle_stop() {
        log::info!("string_in_text: stopped");
    }
}

edgeless_function::export!(StringInTextFun);