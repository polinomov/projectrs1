use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add123(a: i32, b: i32) -> i32 {
    a + b*2
}

#[wasm_bindgen]
pub fn addxx(a: i32, b: i32) -> i32 {
    a + b*2
}


#[wasm_bindgen]
pub fn modify_bytes(mut data: &mut [u8]) {
    for b in data.iter_mut() {
        *b = *b + 1; // modify in place
    }
}

