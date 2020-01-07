use recipe_calculator_lib::pairing::pairing_code_creator;
use recipe_calculator_lib::pairing::pairing_code_creator::PairingCodeCreator;

fn main() {
    let creator = pairing_code_creator::new("".to_owned(), 1, 2, 3).unwrap();
    the_accepting_sync_trait_fn(creator);
}

fn the_accepting_sync_trait_fn(_creator: impl PairingCodeCreator + Sync) {
}