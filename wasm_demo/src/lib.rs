use wasm_bindgen::prelude::*;
use instant::Instant;


#[wasm_bindgen]
pub fn add123(a: i32, b: i32) -> i32 {
    a - b
}

#[wasm_bindgen]
pub fn addxx(a: i32, b: i32) -> i32 {
    a - b
}


#[wasm_bindgen]
pub fn modify_bytes( pixels: &mut [u8]) ->u32 {
    let start = Instant::now();
   // pixels[..].fill(255);

    unsafe {
        let ptr = pixels.as_mut_ptr(); 
        let mut t = 0;
        for i in 0..pixels.len()/4 {
            *ptr.add(t) = 42; 
            *ptr.add(t+1) = 255; 
            *ptr.add(t+2) = 42; 
            *ptr.add(t+3) = 255; 
            t = t + 4;
        }
    }
    
    let elapsed = start.elapsed();
    //1234
    elapsed.as_micros() as u32
}

