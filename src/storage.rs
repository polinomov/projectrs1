
mod pstorage;

pub trait PStorage {
  fn add_point(&mut self, x:f32, y:f32, z:f32, c:u32);
  fn alloc_points(&mut self, num : usize);
}

pub fn make_strorage() -> Box<dyn PStorage> {
  pstorage::make_new_storage()
}
