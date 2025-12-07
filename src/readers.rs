
mod las;
mod xyz;
mod e57;
use crate::storage;


pub trait Seqreader {
  fn start(&self);
  fn next_chunk(&self) -> (u64, u64);
  fn process_bytes(&mut self, _data: &Vec<u8>, pcl: &mut Box<dyn crate::storage::PStorage>);
}



pub fn make_e57() -> Box<dyn Seqreader> {
  e57::make_new_e57()
}

