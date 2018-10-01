extern crate built;

fn main() {
    // Gather build information
    built::write_built_file().expect("Failed to acquire build-time information");
}
