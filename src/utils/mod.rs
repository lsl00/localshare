pub fn check_file_exists(filepath : &str) -> bool{
  std::path::Path::new(filepath).is_file()
}


pub fn get_file_size(filepath : &str) -> (String,u64){
	let pathto = std::path::Path::new(filepath);
	let fd = std::fs::File::open(pathto)
		.expect(&format!("File {} has been deleted or moved.",filepath));
	let meta = fd.metadata().unwrap();

	(
		pathto.file_name().unwrap().to_str().unwrap().to_string(),
		meta.len()
	)
}

// pub struct X {

// }
// impl Drop for X {
//   fn drop(&mut self) {
//     eprintln!("Dropping");
//   }
// }
// pub fn testA(){
//   let x = X{};
  
// }