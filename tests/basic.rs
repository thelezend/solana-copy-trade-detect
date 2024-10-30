use assert_cmd::Command;

mod common;

#[test]
fn test_basic_flow() -> Result<(), Box<dyn std::error::Error>> {
    common::setup();

    let mut cmd = Command::cargo_bin("solana-copy-trade-detect")?;
    cmd.args([
        "-w",
        "C8WtJP4YveQbza5k1otS7BNFQ6My4pjVwecApCEQCNQi",
        "--swap-num",
        "5",
        "--scan-tx-count",
        "30",
    ]);

    let output = cmd.assert().success().get_output().clone();

    // Check if the output is valid JSON
    let output = String::from_utf8(output.stdout)?;
    println!("Output: {}", output);
    serde_json::from_str::<serde_json::Value>(&output).expect("Failed to parse json");
    Ok(())
}
