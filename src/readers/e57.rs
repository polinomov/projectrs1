
//use crate::traits::readers;
// ASTM E2807-11

use crc32c::crc32c;

use roxmltree::{Document, Node};
use std::str::FromStr;
use std::fs::File;
use std::io::Write;

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
  func1: Box<dyn FnMut(&Vec<u8>, &mut JobData) -> Option<Box<PageJob>>>,
  job_next: Option<Box<PageJob>>
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
  if crc32 !=checksum {
    println!("{:X}", crc32); 
    println!("{:X}", checksum); 
  }
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

fn make_vert_job(ofst:u64 , next_j : Option<Box<PageJob>>)-> Option<Box<PageJob>>{
  let ret_job = PageJob {
    jobdata: JobData{
      read_start: (ofst/PAGE_SIZE)*PAGE_SIZE, 
      read_size:PAGE_SIZE, 
      shift:ofst%PAGE_SIZE,
      cnt:0,
      acc:Vec::new()
    },
    func1: Box::new(move |pagedata, jobdata| {
      return None;
    }),
    job_next:next_j
  };

  return Some(Box::new(ret_job));
}

fn collect_vers_job(xml :String) -> Option<Box<PageJob>>{
  let doc = Document::parse(&xml).unwrap();
  let node = doc.root_element();
  let mut num_vert:u32 = 0;
  let mut next_job: Option<Box<PageJob>> = None;
  let mut obj3d :Vec<Node> = Vec::new();
  for child in node.children() {
    if child.tag_name().name() == "data3D" {
      println!("{:?}", child.tag_name().name());
      for vecchild in child.children() {
        if vecchild.tag_name().name() == "vectorChild" {
          //println!("{:?}", vecchild.tag_name().name());
          for pt in vecchild.children() {
            if(pt.tag_name().name() == "points"){
              //println!("{:?}", pt.tag_name().name());
              for attr in pt.attributes() {
                if attr.name() == "recordCount"{
                  let nn = attr.value().parse::<u32>().unwrap();
                  num_vert = num_vert  +  nn;
                }
                //println!("Attribute: {} = {}", attr.name(), attr.value());
              } 
            }
          }
          obj3d.push(vecchild);
          //numObj = numObj + 1;
          //let nj = make_vert_job(0, next_job);
          //next_job = nj;
        }
      }
    }
  }
  println!("POINTS:{:?}", num_vert);
  return next_job;
}

fn blah(xml :String){
  println!("=== BLAH==========");
  //println!("{}",xml);
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
    func1: Box::new(move |pagedata, jobdata: &mut JobData| {
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
      let xml_vec = jobdata.acc.clone();
      let s = String::from_utf8(xml_vec).unwrap();
      //blah(s.clone());
      //println!("{}",s);
      collect_vers_job(s);   
      return None; 
    }),
    job_next:None
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
    }),
    job_next: None
  }; 
  e57.job = Some(Box::new(header_job));
  e57
}











