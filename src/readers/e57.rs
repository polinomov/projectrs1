
//use crate::traits::readers;
// ASTM E2807-11

use crc32c::crc32c;
use std::cmp;
use kiss_xml::dom::*;
use kiss_xml::errors::KissXmlError;
use std::collections::VecDeque;
use std::any::Any;

const PAGE_SIZE: u64 = 1024;

pub struct E57{
  read_start :u64,
  read_size :u64,
  shit_in_page :usize,
  bytes_to_proc :usize,
  pub action: fn(data: &Vec<u8>, obj: &mut E57),
  pub page_size: u64,
  items: Vec<u8>,
  procq: VecDeque<Box<dyn FnMut(&mut E57)>>,
  job: Option<Box<PageJob>>
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

fn read_page(_data: &Vec<u8>,  e57: &mut E57){
  let real_sz :usize = 1020;
  let crc32 = u32::from_le_bytes(_data[1020..1024].try_into().expect("a"));
  let sub_vec = &_data[0..1020];
  let checksum = crc32c(&sub_vec).swap_bytes();
  println!("{:X}", crc32); 
  println!("{:X}", checksum); 
  if let Some(p) = e57.job.as_mut() {
    let first = p.jobdata.shift as usize;
    let next_job = p.call( &_data[first..1020].to_vec());
    if next_job.is_some() {
      e57.job = next_job;
    }
  }
}

struct JobData{
  read_start :u64,
  read_size :u64,
  shift :u64,
  cnt :u32,
  acc: Vec<u8>
}
struct PageJob {
  jobdata: JobData,
  func1: Box<dyn FnMut(&Vec<u8>, &mut JobData) -> Option<Box<PageJob>>>
}

impl PageJob{
  fn call(&mut self, data: &Vec<u8>) -> Option<Box<PageJob>>{ 
    (&mut self.func1)(data, &mut self.jobdata)
  }
}

fn read_header(_data: &Vec<u8>,  obj: &mut E57){
  /* 
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
 // obj.action = read_block;

  let xml_sz   = usize::from_le_bytes(_data[32..40].try_into().expect("a"));
  let xml_rd_job = PageJob {
    read_start: pg_num * obj.page_size,
    cnt:0,
    acc:Vec::new(),
    func1: Box::new(move |pagedata, cnt, acc| {
      *cnt = *cnt + 1;
      println!("{}",cnt);
      0
    }),
  };
  obj.jobs.push(xml_rd_job);
  obj.jobs[0].call(&obj.items);
  obj.jobs[0].call(&obj.items);
 */
  /* 
  let param: u64 = 10;
  obj.procq.push_back(Box::new(move |e57| {
    let pin: u64 = param;
    println!("Closure {}",pin);
    xml2string(e57);
  }));
  */
}

impl crate::readers::Seqreader  for E57{
  fn start(&self) {
    println!("i-am-e57-start");
  }

  fn next_chunk(&self) -> (u64, u64){
    if let Some(j) = self.job.as_ref() {
      return (j.jobdata.read_start, j.jobdata.read_size);
    }
    (0,0) 
  }

  fn process_bytes(& mut self,_data: &Vec<u8>){
     (self.action)(_data, self);
  }
}

fn make_xml_read_job(xml_ofst:u64, xml_size:u64)-> Option<Box<PageJob>>{
  let ret_job = PageJob {
    jobdata: JobData{
      read_start: (xml_ofst/PAGE_SIZE)*PAGE_SIZE, 
      read_size:PAGE_SIZE, 
      shift:xml_ofst%PAGE_SIZE,
      cnt:0,
      acc:Vec::new()
    },
    func1: Box::new(move |pagedata, jobdata| {
      let nbytes = xml_size as usize;
      let pg_len = pagedata.len();
      let bytes_left = nbytes - jobdata.acc.len();
      let bytes_add = if bytes_left>pg_len { pg_len } else { bytes_left };
      jobdata.acc.extend_from_slice(&pagedata[0.. bytes_add]);
      if jobdata.acc.len() < nbytes {
        jobdata.read_start = jobdata.read_start + PAGE_SIZE;
        jobdata.shift = 0;
        return None; // continue reading
      }
      match String::from_utf8(jobdata.acc.clone()) {
        Ok(s) => {
          println!("Valid UTF-8 string: {}", s);
        }
        Err(e) => {
          println!("Invalid UTF-8 data: {:?}", e);
        }
      }
      None
    }),
  };
  Some(Box::new(ret_job))
}

pub fn make_new_e57() -> Box<dyn  crate::readers::Seqreader> {
  let mut e57 = Box::new(E57{ 
    read_start:0,
    read_size:48, 
    action:read_page, 
    page_size:0,
    shit_in_page:0,
    bytes_to_proc:0,
    items: Vec::new(),
    procq: VecDeque::new(),
    job:None
  });

  let header_job = PageJob {
    jobdata: JobData{read_start:0, read_size:1024,shift:0,cnt:0,acc:Vec::new()},
    func1: Box::new(move |pagedata, _j| {
      let s = String::from_utf8(pagedata[..8].to_vec()).expect("Invalid UTF-8");
      println!("string: {}", s);//ASTM-E57
      let xml_ofst = u64::from_le_bytes(pagedata[24..32].try_into().expect("a"));
      let xml_size = u64::from_le_bytes(pagedata[32..40].try_into().expect("a"));
      make_xml_read_job(xml_ofst,xml_size)
    })
  }; 
  e57.job = Some(Box::new(header_job));
  e57
}











