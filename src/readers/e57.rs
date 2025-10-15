
//use crate::traits::readers;
 
pub struct E57{
  pub first: u32,
  pub size: u32,
  pub action: fn(data: &Vec<u8>, obj: &mut E57),
  pub xml_size: u32
}

fn read_xml(_data: &Vec<u8>,  obj: &mut E57){
  let d = _data.clone();

 

  
  let s1 = String::from_utf8_lossy(&d).into_owned();
  println!("Converted string: {}", s1);

  match String::from_utf8(d) {
        Ok(text) => println!("Converted string: {}", text),
        Err(err) => {
            println!("Conversion failed: {}", err);
            let invalid_bytes = err.into_bytes(); // recover original Vec<u8>
            println!("Original bytes: {:?}", invalid_bytes);
        }
  }
  
  
  obj.first = 0;
  obj.size  = 0;
}

fn read_header(_data: &Vec<u8>,  obj: &mut E57){
  let s = String::from_utf8(_data[..8].to_vec()).expect("Invalid UTF-8");

  println!("string: {}", s);//ASTM-E57
  let major_ver = u32::from_le_bytes(_data[8..12].try_into().expect("a"));
  let minor_ver = u32::from_le_bytes(_data[12..16].try_into().expect("a"));
  let xml_ofst:  u64 = u64::from_le_bytes(_data[24..32].try_into().expect("a"));
  let xml_sz:    u64 = u64::from_le_bytes(_data[32..40].try_into().expect("a"));
  obj.action = read_xml;
  obj.first = xml_ofst as u32;
  obj.size = xml_sz as u32;
  println!("xml_sz: {}", obj.size);
}

impl crate::readers::Seqreader  for E57{
  fn start(&self) {
    println!("i-am-e57-start");
  }

  fn next_chunk(&self) -> (u32, u32){
    (self.first, self.size) 
  }

  fn process_bytes(& mut self,_data: &Vec<u8>){
    //let mut me57 = self;
    (self.action)(_data, self);
    //let first_bytes: [u8; 4] = _data[0..4].try_into().expect("a");
    //let first_num = u32::from_le_bytes(_data[0..4].try_into().expect("a"));

    //let major: u32::from_le_bytes(_data[8..12].try_into().internal_err(WRONG_OFFSET)?),
 
  }
}

pub fn make_new_e57() -> Box<dyn  crate::readers::Seqreader> {
  Box::new(E57{first:0 ,size:48, action:read_header, xml_size:0})
}
 









