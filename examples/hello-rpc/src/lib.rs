use bindings::hello_rpc;

struct Component;

impl hello_rpc::HelloRpc for Component {
    fn add(left: u32, right: u32) -> u32 {
        left + right
    }
}

bindings::export!(Component);
