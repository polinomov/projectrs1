use std::fs::File;
use std::path::Path;
use std::io::{self, Read, Seek, SeekFrom};


mod readers;
mod storage;

struct ReaderCbImpl;


impl readers::ReaderCb  for ReaderCbImpl{
  fn message(&mut self, msg : &str){
    println!("{}",msg);
  }
  fn add_point(&mut self, _x:f32, _y:f32, _z:f32, _c:u32){

  }
  fn alloc_points(&mut self, _num : usize){

  }
}



fn main() {
  let path = Path::new("hello.txt");
  let display1 = path.display();
  // /mnt/c/DEV/lasdata/e57/big/
  // Open the path in read-only mode, returns `io::Result<File>`
  let mut file = match File::open("data/bunnyFloat.e57") {
  //let mut file = match File::open("data/StBarthelemy.e57") {
  //let mut file = match File::open("/mnt/c/DEV/lasdata/e57/big/cloud_0.e57") {
    Err(why) => panic!("couldn't open {}: {}", display1, why),
    Ok(file) => file,
  };
  
  let mut rdresp: Box<dyn readers::ReaderCb>  = Box::new(ReaderCbImpl);
  let mut rdobj = readers::make_e57();
  rdobj.start();
  loop{
    let (first, size) = rdobj.next_chunk();
    if size == 0 {
      break;
    }
    {
      let _ = file.seek(SeekFrom::Start(first as u64));
      let mut buffer = vec![0u8; size as usize];
      let _ = file.read_exact(&mut buffer);
      let _ = rdobj.process_bytes(&buffer, &mut rdresp);
    }
  }
  
}