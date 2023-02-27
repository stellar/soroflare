use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

const WASM_PATH: &str = "../";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    delete_old();

    embedd_contract("ASTEROIDS_ENGINE", "game_engine.wasm");
}

fn delete_old() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let contracts_path = Path::new(&out_dir).join("embedded_contracts.rs");
    let html_path = Path::new(&out_dir).join("embedded_html.rs");
    let _ = fs::remove_file(contracts_path);
    let _ = fs::remove_file(html_path);
}

fn embedd_contract(var_name: &str, wasm_file_name: &str) {
    let wasm_path = Path::new(&WASM_PATH).join(wasm_file_name);

    println!("cargo:rerun-if-changed={wasm_path:?}");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("embedded_contracts.rs");

    let raw_wasm = fs::read(&wasm_path);

    if let Err(e) = raw_wasm {
        eprintln!("Error reading WASM file {wasm_path:?}! {e:?}");
    } else if let Ok(wasm) = raw_wasm {
        let wasm_as_string = wasm
            .clone()
            .into_iter()
            .map(|i| format!("0x{i:X},"))
            .collect::<String>();

        let new_line = format!(
            "pub const {} : [u8; {}] = [{}];",
            var_name,
            wasm.len(),
            wasm_as_string
        );

        //mkdirs
        if !dest_path.exists() {
            let _ = fs::write(&dest_path, "");
        }

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&dest_path)
            .unwrap();
        if let Err(e) = writeln!(file, "{new_line}") {
            eprintln!("Failed to embed {wasm_file_name} from wasm! {e:?}");
        }
    }
}
