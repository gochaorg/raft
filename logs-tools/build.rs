use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let dest_path = Path::new(&out_dir).join("build-env.txt");
    let mut f = File::create(&dest_path).unwrap();

    env::vars().for_each(|(var_name, var_value)| {
        writeln!(f,"{var_name}={var_value}").unwrap();
    });

    // f.write_all(b"
    //     pub fn message() -> &'static str {
    //         \"Hello, World!\"
    //     }
    // ").unwrap();
}