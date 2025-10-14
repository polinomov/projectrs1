
mod e57;
mod las;
mod xyz;
//use crate::readers;

pub trait Seqreader {
  fn start(&self);
  fn next_chunk(&self) -> (u32, u32);
  fn process_bytes(&mut self, _data: &Vec<u8>);
}

pub fn make_e57() -> Box<dyn Seqreader> {
  e57::make_new_e57()
}

