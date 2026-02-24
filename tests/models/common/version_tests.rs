use super::*;

#[test]
fn parse_accepts_prefix_and_missing_parts() {
    let v1 = Version::parse("v1").unwrap();
    assert_eq!(v1.major, 1);
    assert_eq!(v1.minor, 0);
    assert_eq!(v1.patch, 0);

    let v2 = Version::parse("V2.3").unwrap();
    assert_eq!(v2.major, 2);
    assert_eq!(v2.minor, 3);
    assert_eq!(v2.patch, 0);

    let v3 = Version::parse("3.4.5").unwrap();
    assert_eq!(v3.major, 3);
    assert_eq!(v3.minor, 4);
    assert_eq!(v3.patch, 5);
}

#[test]
fn parse_rejects_invalid_inputs() {
    for input in ["", "1.2.3.4", "x.2.3", "1.x.3", "1.2.x"] {
        assert!(
            Version::parse(input).is_err(),
            "expected parse failure for '{input}'"
        );
    }
}

#[test]
fn comparison_handles_prerelease_and_numeric_ordering() {
    let prerelease = Version::new(1, 2, 3, true);
    let stable = Version::new(1, 2, 3, false);
    let newer_patch = Version::new(1, 2, 4, false);

    assert!(stable > prerelease);
    assert!(newer_patch > stable);
}
