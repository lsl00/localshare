use std::collections::BTreeSet;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{IpAddr, UdpSocket, SocketAddr, TcpStream};
use std::time::Duration;

use crate::share::Info;
use crate::share::sharehandler::Query;


#[derive(Debug)]
pub struct SingleFile {
  pub ip_addr : IpAddr,
  pub port : u16,
  pub name : String,
  pub index : u64,
  pub size : u64,
  pub sharer : String,
}
impl Display for SingleFile {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f,"{}({}) {} {}bytes",self.sharer,self.ip_addr.to_string(),self.name,self.size)
  }
}

pub fn get_infos() -> std::io::Result<Vec<SingleFile>> {
  let mut result : Vec<SingleFile> = Vec::new();
  let mut bt = BTreeSet::<SocketAddr>::new();

  let sock = UdpSocket::bind("0.0.0.0:39933")?;
  sock.set_nonblocking(true)?;
  let mut buf = [0 as u8;2048];

  let begin_time = std::time::SystemTime::now();
  loop {
    if let Ok(elapsed) = begin_time.elapsed() {
      if elapsed.as_millis() > 200 {
        return Ok(result);
      }
    }
    let (bytes_recv,ipaddr) = match sock.recv_from(&mut buf){
      Ok(x) => x,
      Err(_) => {std::thread::sleep(Duration::from_millis(10));continue;}
    };
    if bt.contains(&ipaddr){
      continue;
    }
    let info = serde_json::from_slice::<Info>(&buf[0..bytes_recv]).unwrap();
    for (i,x) in info.infos.iter().enumerate() {
      result.push(SingleFile{
        ip_addr : ipaddr.ip(),
        port : info.port,
        name : x.0.clone(),
        index : i as u64,
        size : x.1,
        sharer : info.sharer.clone(),
      })
    }
  }
}


pub fn recv(mut f : File,sf : &SingleFile){
  let query : Query = Query{
    index : sf.index,
    start : 0,
    length : sf.size
  };
  let query = serde_json::to_string(&query).unwrap();

  let mut sock = TcpStream::connect((sf.ip_addr,sf.port)).unwrap();
  let mut buf = [0 as u8;2048];
  sock.write(query.as_bytes());
  while let Ok(n) = sock.read(&mut buf) {
    if n == 0 {
      break;
    }
    let _ = f.write(&buf[0..n]);
  }
}