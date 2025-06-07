use wasm_bindgen::prelude::*;
use std::cell::RefCell;

thread_local! {
    static LOG_BUFFER: RefCell<String> = RefCell::new(String::new());
}

#[wasm_bindgen]
pub fn clear_log() {
    LOG_BUFFER.with(|buf| buf.borrow_mut().clear());
}

#[wasm_bindgen]
pub fn get_log() -> String {
    LOG_BUFFER.with(|buf| buf.borrow().clone())
}

pub fn log_message(msg: &str) {
    LOG_BUFFER.with(|buf| buf.borrow_mut().push_str(msg));
    LOG_BUFFER.with(|buf| buf.borrow_mut().push('\n'));
}