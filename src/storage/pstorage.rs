pub struct PStorageImpl{
  num :u64
}


impl crate::storage::PStorage for PStorageImpl{
  fn add_point(&mut self, x:f32, y:f32, z:f32, c:u32){}
  fn alloc_points(&mut self, num : usize){
    println!("alloc points {}", num);
  }
}

pub fn make_new_storage() -> Box<dyn crate::storage::PStorage> {
  let a = Box::new(PStorageImpl{num:0});
  a
}


