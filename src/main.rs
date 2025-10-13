fn foo(_x: &'static str) -> &'static str {
    "world"
}

fn main() {
    let world = "world";
    println!("Hello {}!", world);
}
