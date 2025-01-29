mod bindings;

use crate::bindings::exports::pa_ck::na_me_exports::component_name_api::Guest;
use bindings::wasi::http::types::{
    Fields, IncomingRequest, OutgoingBody, OutgoingResponse, ResponseOutparam,
};
// Import for using common lib:
// use common_lib::example_common_function;

struct Component;

impl bindings::exports::wasi::http::incoming_handler::Guest for Component {
    fn handle(_request: IncomingRequest, outparam: ResponseOutparam) {
        let hdrs = Fields::new();
        let resp = OutgoingResponse::new(hdrs);
        resp.set_status_code(200).unwrap();

        ResponseOutparam::set(outparam, Ok(resp));
    }
}

bindings::export!(Component with_types_in bindings);
