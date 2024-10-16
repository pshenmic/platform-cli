use anyhow::Context;
use dpp::util::entropy_generator::EntropyGenerator;
use getrandom::getrandom;


pub struct MyDefaultEntropyGenerator;

impl EntropyGenerator for MyDefaultEntropyGenerator {
    fn generate(&self) -> anyhow::Result<[u8; 32]> {
        let mut buffer = [0u8; 32];
        getrandom(&mut buffer).context("generating entropy failed").unwrap();
        Ok(buffer)
    }
}