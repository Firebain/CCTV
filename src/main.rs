mod soap;

use soap::prelude::*;

fn main() {
    let probe = soap::ProbeBuilder::new("Device", "d5057fa7-5194-46c6-84ee-c19f0d5e96e9").build().unwrap();
    let _method = soap::MethodBuilder::new().build().unwrap();

    println!("{}", probe);
}