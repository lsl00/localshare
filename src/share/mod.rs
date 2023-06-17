pub mod sharehandler;

use std::{net::UdpSocket, time::Duration, thread::JoinHandle, mem, sync::mpsc::RecvTimeoutError,};
use std::io::{Error,ErrorKind};
use std::io;
use std::sync::mpsc;
use crate::utils::{check_file_exists, get_file_size};

use self::sharehandler::sharehandler;

pub struct Task<T>{
  send : Option<mpsc::Sender<()>>,
  handler : JoinHandle<T>,
}
impl<T : Send + 'static> Task<T> {
  fn new<F>(f : F) -> Task<T>
  where F : FnOnce(mpsc::Receiver<()>) -> T + Send + 'static {
    let (send,recv) = mpsc::channel();
    let handler = std::thread::spawn(move || f(recv));
    Task{
      send : Some(send),
      handler
    }
  }

  fn stop(mut self){
    mem::drop(self.send.take());
    let _ = self.handler.join();
  }
}

pub struct ShareTask{
  filepaths : Vec<String>,
  broadcast_task : Option<BroadcastTask>,
  mstak : Option<Task<io::Result<()>>>,
  info : Info,
  port : u16,
}



impl ShareTask {
  pub fn new(filepaths : Vec<String>,port: u16,sharer : String) -> std::io::Result<ShareTask> {
    for (i,x) in filepaths.iter().enumerate() {
      //Check all provided files exist
      if check_file_exists(x) == false{
        return Err(Error::new(
          ErrorKind::NotFound,
          format!("{}-th file {} doesn't exist.",i,x)
        ));
      }
    }
    
    let info = Info::new(&filepaths, sharer, port);
    // let broadcast_task = BroadcastTask::new(serde_json::to_string(&info)?, random())?;
    Ok(ShareTask{
      filepaths,
      broadcast_task : None,
      mstak : None,
      info,
      port
    })
  }

  pub fn run(&mut self) -> std::io::Result<()>{
    self.broadcast_task = Some(BroadcastTask::new(serde_json::to_string(&self.info)?));
    self.mstak = {
      let fps = self.filepaths.clone();
      let port = self.port;
      Some(Task::new(move |recv| sharehandler(fps, port, recv)))
    };
    Ok(())
  }

  pub fn stop(&mut self){
    self.mstak.take().map(|t| t.stop());
    self.broadcast_task.take().map(|t| t.stop());
  }
}


#[derive(serde::Serialize,serde::Deserialize)]
pub struct Info{
  pub infos: Vec<(String,u64)>,
  pub sharer : String,
  pub port : u16,
}
impl Info {
  pub fn new(filepaths : &Vec<String>,sharer : String,port : u16) -> Info{
    Info{
      infos : filepaths.iter()
        .map(|s| get_file_size(s))
        .collect(),
      sharer,
      port,
    }
  }
}





pub struct BroadcastTask{
  // send: Option<mpsc::Sender<()>>,
  // handler : JoinHandle<Result<(), std::io::Error>>,
  t : Task<std::io::Result<()>>
}

impl BroadcastTask {
  pub fn new(info : String) -> BroadcastTask{
    BroadcastTask{
      t : Task::new(move |recv| broadcast(info, recv))
    }
  }
  pub fn stop(self){
    self.t.stop();
  }
}

pub fn broadcast(info : String,recv : mpsc::Receiver<()>)->std::io::Result<()>{
  //use recv to stop thread
  let sock = UdpSocket::bind("0.0.0.0:0");
  let sock = match sock {
    Ok(s) => s,
    Err(e) =>{
      if cfg!(debug_assertions) {
        eprintln!("Can't bind due to {}",e);
      }
      return Err(e);
    }
  };
  sock.set_broadcast(true)?;
  loop {
    match sock.send_to(info.as_bytes(), "255.255.255.255:39933"){
      Ok(_) => (),
      Err(e) => {
        eprintln!("Broadcast failed due to {}",e);
      }
    }
    
    let should_end = recv.recv_timeout(Duration::from_millis(200));
    match should_end {
      Err(e) => {
        if let RecvTimeoutError::Disconnected = e {
          // eprintln!("{:?} : Broadcast end.{}",std::time::SystemTime::now(),info);
          // mem::drop(sock);
          return Ok(());
        }
      },
      _ => (),
    }
  }
}

