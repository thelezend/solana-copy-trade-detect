use assert_cmd::Command;

mod common;

#[test]
fn basic() -> Result<(), Box<dyn std::error::Error>> {
    common::setup();

    let mut cmd = Command::cargo_bin("solana-copy-trade-detect")?;
    cmd.args([
        "-w",
        "E5nrNt3zGXwin6KvKsDqesPoou1GAbxZ2WdPHTF4q9if",
        "--swap-num",
        "5",
        "--scan-tx-count",
        "30",
    ]);

    let output = cmd.assert().success().get_output().clone();
    println!("{:#?}", output);

    // Check if the output is valid JSON
    let output = String::from_utf8(output.stdout)?;
    serde_json::from_str::<serde_json::Value>(&output).expect("Failed to parse json");
    Ok(())
}
