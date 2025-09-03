use edgeless_function::*;

struct PerfTestFun;


impl EdgeFunction for PerfTestFun {

    // ------ EDGELESS FUNCTIONS REDEFINITION ------
    fn handle_cast(_src: InstanceId, _encoded_message: &[u8]) {
        let id = 1;

        cast("metric", format!("function:begin:{}", id).as_bytes());

        log::info!("handle_cast() called: starting doing computationally heavy stuff...");
    
        let counter: u64 = 1000000;
        let mut init_str = String::new();
        for i in 0..counter {  
            init_str = format!("{}_{}", init_str, i);
        }

        cast("metric", format!("function:end:{}", id).as_bytes());
    }

    fn handle_call(_src: InstanceId, _encoded_message: &[u8]) -> CallRet {
        log::info!("handle_call() called");
        CallRet::NoReply
    }

 fn handle_init(_payload: Option<&[u8]>, _init_state: Option<&[u8]>) {
        let id = 1;
        cast("metric", format!("function:begin:{}", id).as_bytes());

        edgeless_function::init_logger();

        log::info!("Started. Waiting 5000 ms...");

        delayed_cast(5000, "self", b"");          // Action happens in handle_cast()
    }

    fn handle_stop() {
        log::info!("stopped");
    }
}

edgeless_function::export!(PerfTestFun);
