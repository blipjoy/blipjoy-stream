use eater::EaterSim;
use std::env;
use std::fs;

fn main() -> Result<(), std::io::Error> {
    let path = env::args_os().nth(1);
    if path.is_none() {
        todo!("Error handling for cli args");
    }
    let path = path.unwrap();

    let mut sim = EaterSim::new();
    sim.load(&fs::read(path)?);
    sim.run();

    Ok(())
}
