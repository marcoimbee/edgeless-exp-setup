use edgeless_function::*;
use log;

struct SensorSimulatorAddFunction;

impl EdgeFunction for SensorSimulatorAddFunction {
    fn handle_cast(src: InstanceId, _encoded_message: &[u8]) {
        let value = 12;
        log::info!(
            "sensor_simulator_add {:?}:{:?}, new value generated: {}",
            src.node_id,
            src.component_id,
            value
        );
        cast(&"output", format!("{}", value).as_bytes());
    }

    fn handle_call(_src: InstanceId, _encoded_message: &[u8]) -> CallRet {
        CallRet::NoReply
    }

    fn handle_init(payload: Option<&[u8]>, _serialized_state: Option<&[u8]>) {
        edgeless_function::init_logger();
        cast("self", b"");
    }

    fn handle_stop() {
        // noop
    }
}

edgeless_function::export!(SensorSimulatorAddFunction);
