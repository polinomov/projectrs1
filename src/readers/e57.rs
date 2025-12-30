
//use crate::traits::readers;
// ASTM E2807-11

use crc32c::crc32c;

use roxmltree::{Document, Node};
use std::collections::VecDeque;
use std::ops::Range;

const PAGE_SIZE: u64 = 1024;

type PCloud = Box<dyn crate::readers::ReaderCb>;

struct JobData{
  read_start :u64,
  read_size :u64,
  shift :u64,
  cnt :u32,
  acc: Vec<u8>,
  is_done :bool
}

impl JobData{
  fn empty() -> Self {
    Self { read_start:0, read_size:0, shift:0, cnt:0, acc:Vec::new(), is_done:true } 
  }

  fn create(ofst : u64) -> Self {
    println!("job ofst = {}",ofst);
    Self { 
      read_start: (ofst/PAGE_SIZE)*PAGE_SIZE, 
      read_size: PAGE_SIZE, 
      shift: ofst%PAGE_SIZE,
      cnt: 0,
      acc:Vec::new(),
      is_done:true     
    } 
  }

  fn get_f32_def(bits :&Range<usize>, _scale :f32, _shift:f32, data: &Vec<u8> ) -> f32 {

    ////////////////////
    //let f: f32 = 3.1415;
    //let bt = f.to_le_bytes();
    //let tst = f32::from_le_bytes(bt[0..4].try_into().expect("a"));
    //println!("tst = {}",tst);
    ///////////////
    //for bt in data {
    //  print!("{:02X} ", bt);
    //}

    let  tbits : Range<usize> = bits.start/8..bits.end/8;
    let ret = f32::from_le_bytes(data[tbits].try_into().expect("a"));
    println!("ret = {}",ret);
    //println!("{:.4}", ret);
    return ret;
  }

  fn get_i32_def(bits :&Range<usize>, _scale :f32, _shift:f32, data: &Vec<u8> ) -> f32 {
    println!("i32 {} {}",bits.start,bits.end);
    //let ret = i32::from_le_bytes(data[bits].try_into().expect("a"));
    return 0.0;
  }

  fn get_noop_def(_bits :&Range<usize>, _scale :f32, _shift:f32, data: &Vec<u8> ) -> f32 {
    0.0
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
      if jc.1.is_done == false{
        jc.1.shift = 0; 
        self.jqueue.push_front(jc); // continue
      }
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
      return (j.1.read_start, j.1.read_size);
    }
    (0,0) 
  }

  fn process_bytes(& mut self,data: &Vec<u8>, pcl: &mut PCloud){
    (self.action)(data, pcl, self);
  }
}

/////////////// Jobs ////////////////////////////
fn parse_verts_job(ofst :u64, recs:u64, proto:Node<'_, '_>) -> Vec<(PageJob,JobData)>{
  struct RecI{
    brange : Range<usize>,
    get_val: fn(bits :&Range<usize>, _scale :f32, _shift:f32, data: &Vec<u8> ) -> f32,
  }
  impl RecI{
    pub fn def() -> Self{
      Self{brange:(0..0), get_val: JobData::get_noop_def}
    }
  }

  enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
    I = 3,
    E = 4,
    __Count
  }
  const NUM_ITEMS :usize = Axis::__Count as usize;
  struct Record{
    items : [RecI;NUM_ITEMS],
    num_proc : u64
  } 
  impl Record{
    fn new() -> Self {
      Self { 
        items: std::array::from_fn(|_| RecI::def()),
        num_proc: 0
      } 
    } 
  }
  let mut this_rec = Record::new();

  let mut fbit: usize = 0;
  for prec in proto.children() {
    let mut curr = &mut this_rec.items[Axis::E as usize];
    match  prec.tag_name().name() {
      "cartesianX" => curr = &mut this_rec.items[Axis::X as usize],
      "cartesianY" => curr = &mut this_rec.items[Axis::Y as usize],
      "cartesianZ" => curr = &mut this_rec.items[Axis::Z as usize],
      "cartesianInvalidState" => curr = &mut this_rec.items[Axis::I as usize] ,
      _ => println!(" unknown {}", prec.tag_name().name())
    }
     
    //for attr in prec.attributes(){
      if let Some(v_type) = prec.attribute("type") {
        //println!("{} {}", attr.name(), v_type);
        match v_type{
          "Float" => {
            if let Some(v_precision) = prec.attribute("precision") {
              if v_precision == "single" {
                curr.get_val = JobData::get_f32_def;
                curr.brange = fbit..fbit + 32 ;
                fbit = fbit + 32;
              }
            }
          },
          "Integer" => {
           if let ( Some(v_min), Some(v_max)) = (prec.attribute("minimum"), prec.attribute("maximum")) {
              let vmin :u64 = v_min.parse().unwrap();
              let vmax :u64 = v_max.parse().unwrap();
              curr.get_val = JobData::get_i32_def;
              curr.brange = fbit..fbit + 1 ;
              fbit = fbit + 1;
            }
          },
           _ => println!("Unknown type"),
        }
      }
    //}
  }
   
  let ret_job = PageJob {
    exefunc: Some(Box::new(move |pagedata:&Vec<u8>, _j: &mut JobData, pcl: &mut PCloud| {
      let mut bit_shift = 0;//fbit;
      loop {
        let mut rets: [f32; NUM_ITEMS] = [0.0; NUM_ITEMS];
        for a in 0..NUM_ITEMS {
          let br :Range<usize> =  this_rec.items[a].brange.start+bit_shift..this_rec.items[a].brange.end+bit_shift;
          rets[a] = (this_rec.items[a].get_val)(&br, 1.0, 1.0, pagedata);
        }
        pcl.add_point(rets[Axis::X as usize], 
                      rets[Axis::Y as usize],  
                      rets[Axis::Z as usize], 
                      0);
        this_rec.num_proc =  this_rec.num_proc + 1;
        bit_shift  = bit_shift + fbit;
      }
      return vec![( PageJob::empty(),JobData::empty())];//stop
    })),
  }; 

  return vec![(ret_job, JobData::create(ofst))];
}

fn parse_xml_job(xml :String)-> Vec<(PageJob,JobData)>{
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
  let mut jvec:Vec<(PageJob,JobData)>  = Vec::new();
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
    //for prec in proto.children() {
     // println!("{:?}", prec.tag_name().name());
   // }
    let vjob = parse_verts_job(ofst,recs,proto);
    for v in vjob{
      jvec.push(v);
    }
  }
  jvec.push(( PageJob::empty(),JobData::empty())); //stop
  return jvec;
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
        jobdata.is_done = false;
        return Vec::new(); // continue reading
      }
      
      let xml_vec = jobdata.acc.clone();
      let s = String::from_utf8(xml_vec).unwrap();
      println!("{}",s);
      parse_xml_job(s)  
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
      _pcl.message("AAA");
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











