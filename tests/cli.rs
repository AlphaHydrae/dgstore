use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn it_works() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("dgstore")?;

    cmd.arg("LICENSE.txt");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("LICENSE.txt"));

    Ok(())
}
