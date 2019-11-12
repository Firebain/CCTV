mod soap;
mod discovery;

fn main() {
    let probe_matches = discovery::discovery();

    dbg!(probe_matches);
}
