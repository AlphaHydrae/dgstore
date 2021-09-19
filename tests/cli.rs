use assert_cmd::prelude::*;
use fs_extra;
use fs_extra::dir::CopyOptions;
use indoc::indoc;
use predicates::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempdir::TempDir;
use walkdir::WalkDir;

#[macro_use]
extern crate maplit;

#[test]
fn store_file_digest() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = set_up()?;

    let mut cmd = Command::cargo_bin("dgstore")?;
    cmd.current_dir(&tmp_dir)
        .arg("over-the-rainbow.txt")
        .env("NO_COLOR", "");

    cmd.assert().success().stdout(predicate::eq(indoc! {"
        ✓ 49735c4 over-the-rainbow.txt (stored digest to over-the-rainbow.txt.sha512)
    "}));

    let actual_state = recursively_read_directory_contents(tmp_dir.path())?;

    let mut expected_state = test_data();
    expected_state.extend(hashmap! {
        String::from("over-the-rainbow.txt.sha512") =>
            String::from("49735c48046038e54ff7fca9afea03aae18eac08b2e395289a867b221e094dfdca0ad6af3be2b2cc2f950a48a1b4fe61d48ed5d612c797116c39d9e82b5a9829")
    });

    assert_eq!(actual_state, expected_state);

    tear_down(tmp_dir)
}

#[test]
fn check_file_digest() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = set_up()?;
    fs::write(
        tmp_dir.path().join("over-the-rainbow.txt.sha512"),
        "49735c48046038e54ff7fca9afea03aae18eac08b2e395289a867b221e094dfdca0ad6af3be2b2cc2f950a48a1b4fe61d48ed5d612c797116c39d9e82b5a9829"
    )?;

    let mut cmd = Command::cargo_bin("dgstore")?;
    cmd.current_dir(&tmp_dir)
        .arg("over-the-rainbow.txt")
        .env("NO_COLOR", "");

    cmd.assert().success().stdout(predicate::eq(indoc! {"
        ✓ 49735c4 over-the-rainbow.txt
    "}));

    let actual_state = recursively_read_directory_contents(tmp_dir.path())?;

    let mut expected_state = test_data();
    expected_state.extend(hashmap! {
        String::from("over-the-rainbow.txt.sha512") =>
            String::from("49735c48046038e54ff7fca9afea03aae18eac08b2e395289a867b221e094dfdca0ad6af3be2b2cc2f950a48a1b4fe61d48ed5d612c797116c39d9e82b5a9829")
    });

    assert_eq!(actual_state, expected_state);

    tear_down(tmp_dir)
}

#[test]
fn detect_file_change() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = set_up()?;
    fs::write(
        tmp_dir.path().join("over-the-rainbow.txt.sha512"),
        "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
    )?;

    let mut cmd = Command::cargo_bin("dgstore")?;
    cmd.current_dir(&tmp_dir)
        .arg("over-the-rainbow.txt")
        .env("NO_COLOR", "");

    cmd.assert().success().stdout(predicate::eq(indoc! {"
        ✗ 49735c4 over-the-rainbow.txt (previous digest was 0000000)
    "}));

    let actual_state = recursively_read_directory_contents(tmp_dir.path())?;

    let mut expected_state = test_data();
    expected_state.extend(hashmap! {
        String::from("over-the-rainbow.txt.sha512") =>
            String::from("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")
    });

    assert_eq!(actual_state, expected_state);

    tear_down(tmp_dir)
}

#[test]
fn hash_multiple_files_matching_pattern() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = set_up()?;

    let mut cmd = Command::cargo_bin("dgstore")?;
    cmd.current_dir(&tmp_dir)
        .arg("**/*.txt")
        .env("NO_COLOR", "");

    cmd.assert().success().stdout(predicate::eq(indoc! {"
        ✓ 921618b hello/world.txt (stored digest to hello/world.txt.sha512)
        ✓ 49735c4 over-the-rainbow.txt (stored digest to over-the-rainbow.txt.sha512)
    "}));

    let actual_state = recursively_read_directory_contents(tmp_dir.path())?;

    let mut expected_state = test_data();
    expected_state.extend(hashmap! {
        String::from("hello/world.txt.sha512") =>
            String::from("921618bc6d9f8059437c5e0397b13f973ab7c7a7b81f0ca31b70bf448fd800a460b67efda0020088bc97bf7d9da97a9e2ce7b20d46e066462ec44cf60284f9a7"),
        String::from("over-the-rainbow.txt.sha512") =>
            String::from("49735c48046038e54ff7fca9afea03aae18eac08b2e395289a867b221e094dfdca0ad6af3be2b2cc2f950a48a1b4fe61d48ed5d612c797116c39d9e82b5a9829")
    });

    assert_eq!(actual_state, expected_state);

    tear_down(tmp_dir)
}

#[test]
fn recursively_hash_all_files_in_directory() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = set_up()?;

    let mut cmd = Command::cargo_bin("dgstore")?;
    cmd.current_dir(&tmp_dir).arg("**/*").env("NO_COLOR", "");

    cmd.assert().success().stdout(predicate::eq(indoc! {"
        ✓ cf83e13 empty (stored digest to empty.sha512)
        ✓ 921618b hello/world.txt (stored digest to hello/world.txt.sha512)
        ✓ 49735c4 over-the-rainbow.txt (stored digest to over-the-rainbow.txt.sha512)
        ✓ 6f47499 random (stored digest to random.sha512)
    "}));

    let actual_state = recursively_read_directory_contents(tmp_dir.path())?;

    let mut expected_state = test_data();
    expected_state.extend(hashmap! {
        String::from("empty.sha512") =>
            String::from("cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"),
        String::from("hello/world.txt.sha512") =>
            String::from("921618bc6d9f8059437c5e0397b13f973ab7c7a7b81f0ca31b70bf448fd800a460b67efda0020088bc97bf7d9da97a9e2ce7b20d46e066462ec44cf60284f9a7"),
        String::from("over-the-rainbow.txt.sha512") =>
            String::from("49735c48046038e54ff7fca9afea03aae18eac08b2e395289a867b221e094dfdca0ad6af3be2b2cc2f950a48a1b4fe61d48ed5d612c797116c39d9e82b5a9829"),
        String::from("random.sha512") =>
            String::from("6f47499789223ed305a19ca1e5050e2c778a0dde0498268b1053ed82334ba44ba79d56d16c9e4c744bc2237798baca2fea39e74c941970b9344d9e64f209f99d")
    });

    assert_eq!(actual_state, expected_state);

    tear_down(tmp_dir)
}

fn set_up() -> Result<TempDir, Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new("dgstore")?;
    let test_data_dir = env::current_dir()?.as_path().join("tests").join("data");

    let copy_options = CopyOptions {
        content_only: true,
        ..Default::default()
    };
    fs_extra::dir::copy(test_data_dir, tmp_dir.path(), &copy_options)?;

    assert_eq!(
        recursively_read_directory_contents(tmp_dir.path())?,
        test_data()
    );

    Ok(tmp_dir)
}

fn tear_down(tmp_dir: TempDir) -> Result<(), Box<dyn std::error::Error>> {
    tmp_dir.close()?;
    Ok(())
}

fn test_data() -> HashMap<String, String> {
    let mut data = HashMap::new();

    data.insert(
        String::from("hello/world.txt"),
        String::from("Hello, World!\n"),
    );
    data.insert(String::from("empty"), String::from(""));
    data.insert(
        String::from("over-the-rainbow.txt"),
        String::from("Somewhere over the rainbow, way up high\n"),
    );
    data.insert(
        String::from("random"),
        String::from("83c2NQPHOOWUh0JEFWRjdeiZVVRwbqRWDcOe6O7zUM1dJNJBAUReMrRUem2oQhhI9F5uIuOpb30m6rITCO9x5Gb5fnwGKYXOtWNE")
    );

    data
}

fn recursively_read_directory_contents(
    path: &Path,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let test_data_state: HashMap<String, String> = WalkDir::new(path)
        .into_iter()
        .filter(|entry| entry.as_ref().map_or(false, |e| e.file_type().is_file()))
        .map(|entry_result| {
            let entry = entry_result.expect("Could not list test data");
            let test_data_file_path = entry.path();

            let test_data_file_contents =
                fs::read_to_string(test_data_file_path.to_string_lossy().to_string())
                    .expect("Could not read test data");

            (
                test_data_file_path
                    .strip_prefix(&path)
                    .expect("Could not list test data")
                    .to_string_lossy()
                    .to_string(),
                test_data_file_contents,
            )
        })
        .collect();

    Ok(test_data_state)
}
