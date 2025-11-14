
//use crate::traits::readers;
// ASTM E2807-11

use crc32c::crc32c;
use std::cmp;
use kiss_xml::dom::*;
use kiss_xml::errors::KissXmlError;
use std::collections::VecDeque;
use std::any::Any;

const PAGE_SIZE: u64 = 1024;

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

pub struct E57{
  pub action: fn(data: &Vec<u8>, obj: &mut E57),
  job: Option<Box<PageJob>>
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


fn collect_vers_job(xml :&String) -> Option<Box<PageJob>>{
  let dom = kiss_xml::parse_str(xml).unwrap();
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
  None
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
          return collect_vers_job(&s);
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
    action:read_page, 
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











