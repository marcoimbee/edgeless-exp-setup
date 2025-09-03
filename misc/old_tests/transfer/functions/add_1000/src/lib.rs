use edgeless_function::*;

struct Add1000Fun;

impl EdgeFunction for Add1000Fun {
    fn handle_cast(_src: InstanceId, encoded_message: &[u8]) {
        let str_message = core::str::from_utf8(encoded_message).unwrap();

        log::info!("ADD_1000: called with '{}'", str_message);

        if let Ok(n) = str_message.parse::<i32>() {
            cast("res_3", format!("{}", n + 1000).as_bytes());
        }
    }

    fn handle_call(_src: InstanceId, _encoded_message: &[u8]) -> CallRet {
        CallRet::NoReply
    }

    fn handle_init(_payload: Option<&[u8]>, _init_state: Option<&[u8]>) {
        edgeless_function::init_logger();
        log::info!("ADD_1000: started");
    }

    fn handle_stop() {
        log::info!("ADD_1000: stopped");
    }
}

edgeless_function::export!(Add1000Fun);
