use crate::asar;

#[test]
fn test_api_version() {
    let apiversion = asar::api_version();
    assert_eq!(apiversion, 303);
}

#[test]
fn test_version() {
    let version = asar::version();
    assert_eq!(version, 10901);
}

#[test]
fn test_math() {
    let result = asar::math("1+1");
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result, 2f64);
}