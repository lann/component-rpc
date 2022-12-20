use bindings::hello_rpc;

struct Component;

impl hello_rpc::HelloRpc for Component {
    fn add(left: u32, right: u32) -> u32 {
        left + right
    }

    fn decode_hex(hex: String) -> Result<u64, String> {
        u64::from_str_radix(&hex, 16).map_err(|err| format!("{err:#}"))
    }
}

bindings::export!(Component);
