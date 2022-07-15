mod lib;
use::std::collections::HashMap;

fn main() {
    let mut mapping: HashMap<String, String> = HashMap::new();
    // master password handling
    lib::master_password();
    // data handling
    lib::saved_data_handling(&mut mapping);
    // handle user input
    lib::input_loop(&mut mapping);
    // restore files to their updated state after close
    lib::restore_files(&mut mapping);
}