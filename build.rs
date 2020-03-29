use std::env;
/*
use std::fs;
use std::io;

const CPP_FILES: [&str; 5] = [
    "cimgui/cimgui.cpp",
    "cimgui/imgui/imgui.cpp",
    "cimgui/imgui/imgui_demo.cpp",
    "cimgui/imgui/imgui_draw.cpp",
    "cimgui/imgui/imgui_widgets.cpp",
];

fn assert_file_exists(path: &str) -> io::Result<()> {
  match fs::metadata(path) {
    Ok(_) => Ok(()),
      Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
        panic!(
          "Can't access {}. Did you forget to fetch git submodules?",
          path
        );
      }
    Err(e) => Err(e),
  }
}*/


use std::process::Command;

fn main() {
  let target = env::var("TARGET").unwrap();
  if target.contains("apple-ios") {
    println!("cargo:rustc-link-search=framework=/Library/Frameworks/");
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=framework=MoltenVK");
    println!("cargo:rustc-link-lib=framework=Metal");
    println!("cargo:rustc-link-lib=framework=IOSurface");
    println!("cargo:rustc-link-lib=framework=QuartzCore");
    println!("cargo:rustc-link-lib=framework=UIKit");
    println!("cargo:rustc-link-lib=framework=Foundation");
  }
  
 // if !(target.contains("windows") || target.contains("apple-ios")) {
   /* let output = Command::new("sh")
            .arg("-c")
            .arg("./src/shaders/glslangValidator")
            .arg("-V")
            .arg("./src/shaders/glsl/VkModel.vert")
            .arg("-o")
            .arg("./src/shaders/sprv/testVkModelVert.spv")
            .output()
            .expect("Failed to run glslangValidator");*/
   // println!("cargo:rerun-if-changed=src/shaders/glsl");
 // }
  
  
  
  
  /*
  
  let mut build = cc::Build::new();
  build.cpp(true);
  for path in &CPP_FILES {
    assert_file_exists(path).expect("Failed to do the things with the imgui build");
    build.file(path);
  }
  build.compile("libcimgui.a");*/
}
