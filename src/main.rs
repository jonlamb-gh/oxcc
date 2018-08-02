// https://github.com/PolySync/oscc/wiki/Hardware-Main
// https://github.com/jonlamb-gh/oscc/

mod fault_condition;
mod pid;
mod throttle_module;

use throttle_module::ThrottleModule;

fn main() {
    println!("Hello, world!");

    let _throttle_module = ThrottleModule::new();

    println!("All done");
}
