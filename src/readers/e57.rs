
//use crate::traits::readers;
// ASTM E2807-11

use crc32c::crc32c;
use std::cmp;
use kiss_xml::dom::*;
use kiss_xml::errors::KissXmlError;
use std::collections::VecDeque;

pub struct E57{
  read_start :u64,
  read_size :u64,
  shit_in_page :usize,
  bytes_to_proc :usize,
  pub action: fn(data: &Vec<u8>, obj: &mut E57),
  pub page_size: u64,
  items: Vec<u8>,
  procq: VecDeque<Box<dyn FnMut(&mut E57)>>
}


fn parse_xml(xml :&String, _obj: &mut E57) -> Result<(), kiss_xml::errors::KissXmlError>{
  let dom = kiss_xml::parse_str(xml)?;
  let root = dom.root_element();
  for data3d in root.elements_by_name("data3D"){
    for str in data3d.elements_by_name("vectorChild"){
      for point in str.elements_by_name("points"){
        // points
        let mut file_ofst :u64 = 0;
        for ptattr in  point.attributes(){         
          println!("{}: {}", ptattr.0, ptattr.1);
          let command: &String = &String::from(ptattr.0);
          match command.as_str() {
            "fileOffset" => file_ofst = ptattr.1.parse().expect("Not a valid u64"),
            "recordCount" => println!("Stopping..."),
            "type" => println!("Pausing..."),
            _ => println!("Unknown command"),
          }
        }
        println!("{}",file_ofst); 
      }//point
    }
  }
  Ok(())
}

fn xml2string(_obj: &mut E57){
  match String::from_utf8(_obj.items.clone()) {
    Ok(text) => {
      println!("XML: {}",text);
      parse_xml(&text, _obj);
    }
    Err(err) => {
      println!("Conversion failed: {}", err);
      let invalid_bytes = err.into_bytes(); // recover original Vec<u8>
      println!("Original bytes: {:?}", invalid_bytes);
    }
  }
}

fn read_block(_data: &Vec<u8>,  obj: &mut E57){
  let real_sz :usize = 1020;
  let crc32 = u32::from_le_bytes(_data[1020..1024].try_into().expect("a"));
  let sub_vec = &_data[0..1020];
  let checksum = crc32c(&sub_vec).swap_bytes();
  println!("{:X}", crc32); 
  println!("{:X}", checksum); 
  let first = obj.shit_in_page;
  let mut last: usize = first + obj.bytes_to_proc as usize;
  last = cmp::min(last, real_sz);
  let mut vv:  Vec<u8>  = _data[first..last].to_vec();
  obj.items.append(&mut vv);
  obj.shit_in_page = 0;
  obj.bytes_to_proc -= last - first;
  if obj.bytes_to_proc == 0 {
    let pfunc= obj.procq.pop_front();
    if let Some(mut f) = pfunc {
        f(obj); 
    } 
    obj.items.clear();
  } else{
    obj.read_start += obj.page_size;
  }
}

fn read_header(_data: &Vec<u8>,  obj: &mut E57){
  let s = String::from_utf8(_data[..8].to_vec()).expect("Invalid UTF-8");
  println!("string: {}", s);//ASTM-E57
  let major_ver = u32::from_le_bytes(_data[8..12].try_into().expect("a"));
  let minor_ver = u32::from_le_bytes(_data[12..16].try_into().expect("a"));
  let xml_ofst:  u64 = u64::from_le_bytes(_data[24..32].try_into().expect("a"));
  obj.bytes_to_proc  = usize::from_le_bytes(_data[32..40].try_into().expect("a")); //xml_size
  obj.page_size = u64::from_le_bytes(_data[40..48].try_into().expect("a"));
  let pg_num :u64 = xml_ofst/obj.page_size;
  obj.read_start = pg_num * obj.page_size;
  obj.shit_in_page = (xml_ofst - obj.read_start) as usize;
  obj.read_size = obj.page_size;
  obj.action = read_block;

  let param: u64 = 10;
  obj.procq.push_back(Box::new(move |e57| {
    let pin: u64 = param;
    println!("Closure {}",pin);
    xml2string(e57);
  }));
}

impl crate::readers::Seqreader  for E57{
  fn start(&self) {
    println!("i-am-e57-start");
  }

  fn next_chunk(&self) -> (u64, u64){
    (self.read_start,self.read_size) 
  }

  fn process_bytes(& mut self,_data: &Vec<u8>){
     (self.action)(_data, self);
  }
}

pub fn make_new_e57() -> Box<dyn  crate::readers::Seqreader> {
  Box::new(E57{ 
    read_start:0,
    read_size:48, 
    action:read_header, 
    page_size:0,
    shit_in_page:0,
    bytes_to_proc:0,
    items: Vec::new(),
    procq: VecDeque::new()
  })
}











