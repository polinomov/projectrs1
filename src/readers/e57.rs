
//use crate::traits::readers;
// ASTM E2807-11

use crc32c::crc32c;

use roxmltree::{Document, Node};
use std::collections::VecDeque;

const PAGE_SIZE: u64 = 1024;

type PCloud = Box<dyn crate::storage::PStorage>;

struct JobData{
  read_start :u64,
  read_size :u64,
  shift :u64,
  cnt :u32,
  acc: Vec<u8>
}

impl JobData{
  fn empty() -> Self {
    Self { read_start:0, read_size:0, shift:0, cnt:0, acc:Vec::new() } 
  }

  fn create(ofst : u64) -> Self {
    Self { 
      read_start: (ofst/PAGE_SIZE)*PAGE_SIZE, 
      read_size: PAGE_SIZE, 
      shift: ofst%PAGE_SIZE,
      cnt: 0,
      acc:Vec::new()     
    } 
  }
}

struct PageJob {
  exefunc: Option<Box<dyn FnMut(&Vec<u8>, &mut JobData, &mut PCloud) -> Vec<(PageJob,JobData)>>>,
}
impl PageJob{
  fn empty() -> Self {
    Self { 
      exefunc: None
    } 
  }
  
  fn execute(& mut self, data: &Vec<u8>,  jd: &mut JobData, pcl: &mut PCloud) -> Vec<(PageJob,JobData)>{
    if let Some(ff) = self.exefunc.as_mut(){
      ff(data, jd, pcl)
    }
    else{
      Vec::new()
    }
  } 
}

pub struct E57{
  pub action: fn(data: &Vec<u8>, pcl: &mut PCloud, obj: &mut E57),
  jqueue : VecDeque<(PageJob,JobData)>,
}

impl E57{
  fn create_job(&mut self,j:PageJob, d: JobData){
    self.jqueue.push_back((j,d));
  }
  fn call_job(&mut self, data: &Vec<u8>, pcl: &mut PCloud){
    let mut jc = self.jqueue.pop_front().unwrap();
    let first = jc.1.shift as usize;
    let ret = jc.0.execute(&data[first..1020].to_vec(), &mut jc.1, pcl);
    if ret.len() == 0 {
      jc.1.shift = 0; 
      self.jqueue.push_front(jc); // continue
    } else{
      for r in ret {
        self.jqueue.push_back(r);
      }
    }
  }
}

fn read_page(_data: &Vec<u8>, pcl: &mut PCloud,  e57: &mut E57){
  let real_sz :usize = 1020;
  let crc32 = u32::from_le_bytes(_data[1020..1024].try_into().expect("a"));
  let sub_vec = &_data[0..1020];
  let checksum = crc32c(&sub_vec).swap_bytes();
  if crc32 !=checksum {
    println!("{:X}", crc32); 
    println!("{:X}", checksum); 
  }
  e57.call_job(_data, pcl);
}

impl crate::readers::Seqreader  for E57{
  fn start(&self) {
    println!("i-am-e57-start");
  }

  fn next_chunk(&self) -> (u64, u64){
    if let Some(j) = self.jqueue.front() {
      //println!("{} {}",j.1.read_start, j.1.read_size);
      return (j.1.read_start, j.1.read_size);
    }
    (0,0) 
  }

  fn process_bytes(& mut self,data: &Vec<u8>, pcl: &mut PCloud){
    //pcl.alloc_points(0);
    (self.action)(data, pcl, self);
  }
}

/////////////// Jobs ////////////////////////////

fn collect_verts_job(xml :String)-> Vec<(PageJob,JobData)>{
  let doc = Document::parse(&xml).unwrap();
  let node = doc.root_element();
  let mut obj3d :Vec<Node> = Vec::new();
  for child in node.children() {
    if child.tag_name().name() == "data3D" {
      for vecchild in child.children() {
        obj3d.push(vecchild);
      }
    }
  }
  for j3d in obj3d {
    let Some(pt_node) = j3d.children().find(|n| n.has_tag_name("points")) else{
      continue;
    };
    let mut recs = 0;
    let mut ofst = 0;
    if let Some(attr) = pt_node.attribute("recordCount") {
      recs = attr.parse::<u64>().unwrap();
    }
    if let Some(attr) = pt_node.attribute("fileOffset") {
      ofst = attr.parse::<u64>().unwrap();
    }
    let Some(proto) = pt_node.children().find(|n| n.has_tag_name("prototype")) else{
      continue;
    };
    for prec in proto.children() {
      println!("{:?}", prec.tag_name().name());
    }
  }
  return vec![( PageJob::empty(),JobData::empty())];//stop
}

fn make_xml_read_job(xml_ofst:u64, xml_size:u64)-> Vec<(PageJob,JobData)> {
  let ret_job = PageJob {
    exefunc: Some(Box::new(move |pagedata, jobdata: &mut JobData, _pcl: &mut PCloud| {
      let nbytes = xml_size as usize;
      let pg_len = pagedata.len();
      let bytes_left = nbytes - jobdata.acc.len();
      let bytes_add = if bytes_left>pg_len { pg_len } else { bytes_left };
 
      jobdata.acc.extend_from_slice(&pagedata[0.. bytes_add]);
      if jobdata.acc.len() < nbytes {
        jobdata.read_start = jobdata.read_start + PAGE_SIZE;
        return Vec::new(); // continue reading
      }
      let xml_vec = jobdata.acc.clone();
      let s = String::from_utf8(xml_vec).unwrap();
      println!("{}",s);
      collect_verts_job(s)  
    }))
  };
  return vec![(ret_job, JobData::create(xml_ofst))];
}

pub fn make_new_e57() -> Box<dyn  crate::readers::Seqreader> {
  let mut e57 = Box::new(E57{ 
    action:read_page, 
    jqueue: VecDeque::new()
  });

  let header_job = PageJob {
    exefunc: Some(Box::new(move |pagedata:&Vec<u8>, _j: &mut JobData, _pcl: &mut PCloud| {
      let s = String::from_utf8(pagedata[..8].to_vec()).expect("Invalid UTF-8");
      println!("string: {}", s);//ASTM-E57
      let xml_ofst = u64::from_le_bytes(pagedata[24..32].try_into().expect("a"));
      let xml_size = u64::from_le_bytes(pagedata[32..40].try_into().expect("a"));
      make_xml_read_job(xml_ofst,xml_size)
    })),
  }; 
  
  e57.create_job(header_job, JobData::create(0));
  e57
}











