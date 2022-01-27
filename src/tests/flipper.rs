use crate::{
    uis::{
        Result,
        Ui,
        Upload,
    },
    utils::{
        self,
        cargo_contract,
    },
};
use lang_macro::waterfall_test;
use crate::tests::{ADDRESSES_FILE, DEPOSIT};
use std::fs::OpenOptions;
use std::io::prelude::*;

#[waterfall_test(example = "flipper")]
async fn upload_flipper(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("flipper/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = ui
        .execute_upload(Upload::new(contract_file).push_initial_value("value", DEPOSIT.to_string().as_str()).caller("ALICE")).await?;


    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(ADDRESSES_FILE)
        .unwrap();

    writeln!(file, "flipper: {}", contract_addr).expect("");
    Ok(())
}
