use std::{
	fs,
	process::Command
};

fn main() {
	let src_dir = "engine/shaders/";
	let dst_dir = "target/shaders/";

	println!("cargo:rerun-if-changed={}", src_dir);

	// Create output directory if it doesn't already exist
	fs::create_dir_all(dst_dir).unwrap();

	// Retrieve all files in the shader source directory
	let files: Vec<String> = fs::read_dir(src_dir).unwrap()
		.map(|e| e.unwrap().path().file_name().unwrap().to_owned().into_string().unwrap())
		.collect();

	for file in files {
		println!("cargo:rerun-if-changed={}", file);

		let mut input_file = src_dir.to_owned();
		input_file.push_str(&file);

		let mut output_file = dst_dir.to_owned();
		output_file.push_str(&file);
		output_file.push_str(".spv");

		let output = Command::new("glslc.exe").args(&[&input_file, "-o", &output_file]).output().unwrap();

		assert!(output.status.success(), "Failed to compile {}\nstatus: {}\nstdout: {}\nstderr: {}",
			input_file,
			output.status,
			String::from_utf8_lossy(&output.stdout),
			String::from_utf8_lossy(&output.stderr));
	}
}