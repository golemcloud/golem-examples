mod bindings;

use std::cell::RefCell;
use crate::bindings::exports::pack::name::api::*;
use crate::bindings::golem::api::host::*;

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};

/// This is one of any number of data types that our application
/// uses. Golem will take care to persist all application state,
/// whether that state is local to a function being executed or
/// global across the entire program.
struct State {
    total: u64,
}

/// This holds the state of our application.
thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        total: 0,
    });
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RequestBody {
    current_total: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ResponseBody {
    message: String,
}

struct Component;

impl Guest for Component {
    /// Updates the component's state by adding the given value to the total.
    fn add(value: u64) {
        STATE.with_borrow_mut(|state| state.total += value);
    }

    /// Returns the current total.
    fn get() -> u64 {
        STATE.with_borrow_mut(|state| state.total)
    }

    /// Sends the current total to a remote server's REST API
    fn publish() -> Result<(), String> {
        STATE.with_borrow_mut(|state| {
            println!("Publishing the total count {} via HTTP", state.total);
            let client = Client::builder().build()?;

            let request_body = RequestBody { current_total: state.total };
            let response: Response = client.post("http://localhost:9999/current-total")
                .json(&request_body)
                .send()?;

            let response_body = response.json::<ResponseBody>()?;
            println!("Result: {:?}", response_body);

            Ok(())
        }).map_err(|e: reqwest::Error| format!("Failed to publish: {}", e))
    }

    /// Pauses the component until a Promise is fulfilled externally
    fn pause() {
        let promise_id = golem_create_promise();
        golem_await_promise(&promise_id);
    }
}
