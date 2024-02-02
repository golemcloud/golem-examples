mod bindings;

use crate::bindings::exports::pack::name::api::*;
use lib::core;

struct AppState(usize);

static mut APP_STATE: AppState = AppState(0);

fn with_app_state<T>(f: impl FnOnce(&mut AppState) -> T) -> T {
    unsafe { f(&mut APP_STATE) }
}

struct Component;

impl Guest for Component {
    fn hello() -> String {
        with_app_state(|state| {
            let (n, message) = core::hello(state.0);

            state.0 = n;

            message
        })
    }
}
