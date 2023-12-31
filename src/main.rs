use std::fs::File;
use std::time::Duration;

use share::ShareTask;

use crate::receive::recv;
use crate::utils::check_file_exists;

pub mod share;
pub mod utils;
pub mod receive;
fn main() {
  let files : Vec<String> = std::env::args().skip(1).collect();
	// let mut task = ShareTask::new(files,4399,"lsl".to_string()).unwrap();
  // task.run();
  if files.len() == 0 {
    let info = receive::get_infos().unwrap();
    for (i,x) in info.iter().enumerate() {
      println!("{}. {}",i,x)
    }
    eprint!("Which one? ");
    let which = loop{
      let mut line = String::new();
      let _ = std::io::stdin().read_line(&mut line);
      if let Ok(u) = line.trim().parse::<usize>() {
        if u < info.len(){
          break u;
        }else{
          eprintln!("Invalid input");
        }
      }else {
        eprintln!("Invalid input");
      }
    };
    let mut line = String::new();
    loop {
      line.clear();
      eprint!("Enter a path with file name:");
      std::io::stdin().read_line(&mut line);
      if line.trim().len() == 0 {
        line = info[which].name.clone();
      }
      if check_file_exists(line.trim()) {
        eprint!("Override?(y/n)");
        let mut tmp = String::new();
        std::io::stdin().read_line(&mut tmp);
        if tmp.trim().starts_with('y') == false{
          continue;
        }
      }
      
      if let Ok(fd) = File::options().write(true).create(true).open(line.trim()) {
        recv(fd, &info[which]);
        break;
      }else{
        eprintln!("Invalid filename")
      }
    }
  }else{
    let mut t = ShareTask::new(files, 4399, "SomeBody".to_string()).unwrap();
    t.run().unwrap();
    loop{
      std::thread::sleep(Duration::from_secs(1000));
    }
  }
}
