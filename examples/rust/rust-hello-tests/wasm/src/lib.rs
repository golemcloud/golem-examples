use bindings::*;
use exports::pack::name::api::*;
use lib::core;

struct AppState(usize);

static mut APP_STATE: AppState = AppState(0);

fn with_app_state<T>(f: impl FnOnce(&mut AppState) -> T) -> T {
    unsafe { f(&mut APP_STATE) }
}

struct ComponentNameImpl;

impl Api for ComponentNameImpl {
    fn hello() -> String {
        with_app_state(|state| {
            let (n, message) = core::hello(state.0);

            state.0 = n;

            message
        })
    }
}

bindings::export!(ComponentNameImpl);
