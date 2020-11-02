use std::{fs, process::Command};

fn main() {
	let src_dir = "src/shaders/";
	let dst_dir = "../target/shaders/";

	println!("cargo:rerun-if-changed={}", src_dir);

	// Create output directory if it doesn't already exist
	fs::create_dir_all(dst_dir).unwrap();

	// Retrieve all files in the shader source directory
	let files: Vec<String> = fs::read_dir(src_dir).unwrap()
		.map(|e| e.unwrap().path().file_name().unwrap().to_owned().into_string().unwrap())
		.collect();

	for file in files {
		println!("cargo:rerun-if-changed={}", file);

		let input_file = format!("{}{}", src_dir, file);
		let output_file = format!("{}{}.spv", dst_dir, file);
		let output = Command::new("glslc.exe").args(&[&input_file, "-o", &output_file]).output().unwrap();

		assert!(output.status.success(), "Failed to compile {}\nstatus: {}\nstdout: {}\nstderr: {}",
			input_file,
			output.status,
			String::from_utf8_lossy(&output.stdout),
			String::from_utf8_lossy(&output.stderr));
	}
}