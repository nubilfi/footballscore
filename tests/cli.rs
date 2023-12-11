use anyhow::Error;
use assert_cmd::{cargo::cargo_bin, Command};
use log::info;

use footballscore::config::TestEnvs;

#[ignore]
#[test]
fn test_default() -> Result<(), Error> {
    let _env = TestEnvs::new(&["API_KEY", "API_ENDPOINT", "CLUB_ID"]);
    let bin = cargo_bin("footballscore");
    assert!(bin.exists());

    let output_live_fixture = Command::cargo_bin("footballscore")?
        .args(["-k", "1e5765fc0c22df4e4ccf20581c2ef3d7", "-c", "529"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output_live_fixture.stdout);
    let stderr = String::from_utf8_lossy(&output_live_fixture.stderr);
    info!("{}", stdout);
    info!("{}", stderr);

    assert!(
        stdout.contains("Error: token - Error/Missing application key. Go to https://www.api-football.com/documentation-v3 to learn how to get your API application key.")
        || stdout.contains("Match: no live event")
    );

    let output_next_fixture = Command::cargo_bin("footballscore")?
        .args(["-k", "1e5765fc0c22df4e4ccf20581c2ef3d7", "-c", "529", "--next-match", "1"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output_next_fixture.stdout);
    let stderr = String::from_utf8_lossy(&output_next_fixture.stderr);
    info!("{}", stdout);
    info!("{}", stderr);

    assert!(
        stdout.contains("Error: token - Error/Missing application key. Go to https://www.api-football.com/documentation-v3 to learn how to get your API application key.")
        || stdout.contains("Barcelona")
    );

    let output_team_information = Command::cargo_bin("footballscore")?
        .args(["-k", "1e5765fc0c22df4e4ccf20581c2ef3d7", "-n", "arsenal"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output_team_information.stdout);
    let stderr = String::from_utf8_lossy(&output_team_information.stderr);
    info!("{}", stdout);
    info!("{}", stderr);

    assert!(
        stdout.contains("Error: token - Error/Missing application key. Go to https://www.api-football.com/documentation-v3 to learn how to get your API application key.")
        || stdout.contains("Here's your club information:")
    );

    Ok(())
}
