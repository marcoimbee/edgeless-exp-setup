use edgeless_function::*;

struct Add10Fun;

impl EdgeFunction for Add10Fun {
    fn handle_cast(_src: InstanceId, encoded_message: &[u8]) {
        let str_message = core::str::from_utf8(encoded_message).unwrap();

        log::info!("ADD_10: called with '{}'", str_message);
	log::info!("ADD_10 is executing on node '{}'", nodeId)

        if let Ok(n) = str_message.parse::<i32>() {
            cast("res_1", format!("{}", n + 10).as_bytes());
        }
    }

    fn handle_call(_src: InstanceId, _encoded_message: &[u8]) -> CallRet {
        CallRet::NoReply
    }

    fn handle_init(_payload: Option<&[u8]>, _init_state: Option<&[u8]>) {
        edgeless_function::init_logger();
        log::info!("ADD_10: started");
    }

    fn handle_stop() {
        log::info!("ADD_10: stopped");
    }
}

edgeless_function::export!(Add10Fun);
