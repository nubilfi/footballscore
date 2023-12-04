use anyhow::Error;
use assert_cmd::{cargo::cargo_bin, Command};
use log::info;

use footballscore::config::TestEnvs;

#[ignore]
#[test]
fn test_default() -> Result<(), Error> {
    let _env = TestEnvs::new(&["API_KEY", "API_ENDPOINT", "CLUB_ID", "API_PATH"]);
    let bin = cargo_bin("footballscore");
    assert!(bin.exists());

    let output = Command::cargo_bin("footballscore")?
        .args(["-k", "1e5765fc0c22df4e4ccf20581c2ef3d7", "-c", "529"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    info!("{}", stdout);
    info!("{}", stderr);

    assert!(stdout.contains("Match: no live event"));

    Ok(())
}
