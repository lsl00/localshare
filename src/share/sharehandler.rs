use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

#[derive(serde::Serialize,serde::Deserialize,Debug)]
pub struct Query{
  pub index : u64,
  pub start : u64,
  pub length : u64,
}

fn handle_connect(mut stream : TcpStream,arc : Arc<Vec<String>>) {
  stream.set_nonblocking(false).unwrap();
  let mut buf = [0 as u8;2048];
  let bytes_read = match stream.read(&mut buf) {
    Ok(n) => n,
    Err(_) => return,
  };
  let query : Query = match serde_json::from_slice(&buf[0..bytes_read]) {
    Ok(q) => q,
    Err(_) => return,
  }; //Get query info
  let total_files = arc.len();
  if query.index as usize >= total_files {
    //Queried file doesn't exist
    eprintln!("Out of bound : {} {}",query.index,total_files);
    return;
  }

  let mut fd = match std::fs::File::open(&arc[query.index as usize]) { //open file
    Ok(fd) => fd,
    Err(e) => {
      eprintln!("Error when open file : {}",e);
      return;
    }
  };
  let _= fd.seek(std::io::SeekFrom::Start(query.start)); //Seek to correct position
  let mut total_res = query.length as usize;
  let mut buf = [0 as u8;2048];
  while total_res > 0{
    //Reading file,sending to socket
    let should_write = total_res.min(1024);
    let should_write = match fd.read(&mut buf[0..should_write]){
      Ok(0) => break,
      Ok(n) => n,
      Err(e) => {
        eprintln!("Error occur while reading file : {}",e);
        return;
      }
    };
    let mut buf_start = 0;
    while buf_start < should_write{
      match stream.write(&buf[buf_start..should_write]) {
        Ok(n) => {total_res -= n;buf_start += n;}
        Err(e) => {
          eprintln!("Error occur while sending:{}",e);
          return;
        }
      }
    }
  }
}

pub fn sharehandler(
  filepaths : Vec<String>,
  port : u16,
  recv : mpsc::Receiver<()>) -> std::io::Result<()>
{
  let sock = TcpListener::bind(("0.0.0.0",port))?;
  sock.set_nonblocking(true)?;

  let arc = Arc::new(filepaths);

  for stream in sock.incoming(){
    let mut handle = false;
    match stream {
      Ok(stream) => {
        match stream.peer_addr(){
          Ok(addr ) => println!("Connect from {}.",addr.to_string()),
          Err(_) => println!("Serving unknown address.")
        }
        handle = true;
        let arc = Arc::clone(&arc);
        std::thread::spawn(move || handle_connect(stream,arc));
      },
      Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {},
      Err(_) => {},
    }

    match recv.try_recv() {
      Err(e) => {
        if let TryRecvError::Disconnected = e{
          return Ok(());
        }
      },
      Ok(_) => {}
    }
    if handle == false{
      std::thread::sleep(Duration::from_millis(50));
    }
  }
  Ok(())
}