extern crate cmake;
use cmake::Config;

fn main()
{
    let dst = Config::new("native/libwinbridge").build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=winbridge");
    println!("cargo:rustc-link-lib=static=user32");
}