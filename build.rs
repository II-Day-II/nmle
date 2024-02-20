use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let shader_dir = Path::new("./src/shaders");
    for shader_file in shader_dir
        .read_dir()
        .expect("Reading shader directory failed")
    {
        if let Ok(file) = shader_file {
            match file.path().extension().and_then(OsStr::to_str) {
                Some("frag") | Some("vert") => {}
                Some("wgsl") => continue,
                Some(e) => {
                    panic!("{}", e);
                }
                None => continue,
            }
            let mut ext_spv = file
                .path()
                .extension()
                .expect("Extension missing")
                .to_os_string();
            ext_spv.push(".spv");

            let filename = file.file_name();
            std::fs::create_dir_all(Path::new(&out_dir).join("shaders"))
                .expect("Failed to create shaders directory");
            let out_file = Path::new(&out_dir)
                .join("shaders")
                .join(filename)
                .with_extension(ext_spv);
            let glslang_validator_output = Command::new("glslangValidator")
                .arg("-V")
                .arg(file.path())
                .arg("-o")
                .arg(out_file)
                .output()
                .expect("failed to run glslangValidator");
            let status = glslang_validator_output.status;
            if !status.success() {
                unsafe {
                    panic!(
                        "glslangValidator: {}\n{}",
                        String::from_utf8_unchecked(glslang_validator_output.stderr),
                        String::from_utf8_unchecked(glslang_validator_output.stdout)
                    );
                }
            }
            println!("glslangValidator exited with code {}", status);
        } else {
            eprintln!("Failed to find a file, {:?}", shader_file);
        }
    }
    println!("cargo:rerun-if-changed=src/shaders");
}
