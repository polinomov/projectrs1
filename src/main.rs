use std::fs::File;
use std::path::Path;
use std::io::Read;

mod readers;

fn main() {
  let path = Path::new("hello.txt");
  let display1 = path.display();

  // Open the path in read-only mode, returns `io::Result<File>`
  let mut file = match File::open("data/my_file.txt") {
    Err(why) => panic!("couldn't open {}: {}", display1, why),
    Ok(file) => file,
  };

  let mut buff: Vec<u8> = Vec::new();
    match file.read_to_end(&mut buff){
      Ok(bytes_read) => {
        println!("Successfully read {} bytes", bytes_read);
      }
      Err(e) => {
        eprintln!("Failed to read file: {}", e);
      }    
  }
  readers::hello_readers();
  //readers::api::say_hello();
}