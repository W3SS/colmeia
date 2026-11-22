use colmeia::sbom::{parse, PackageDep};
use tempfile::tempdir;

#[test]
fn test_parse_cargo_lock() {
    let dir = tempdir().unwrap();
    let lock_content = r#"
version = 3

[[package]]
name = "colmeia"
version = "0.1.0"

[[package]]
name = "rusqlite"
version = "0.31.0"
"#;
    std::fs::write(dir.path().join("Cargo.lock"), lock_content).unwrap();

    let deps = parse(&dir.path().to_string_lossy());
    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&PackageDep {
        ecosystem: "crates.io".into(),
        name: "colmeia".into(),
        version: "0.1.0".into(),
    }));
    assert!(deps.contains(&PackageDep {
        ecosystem: "crates.io".into(),
        name: "rusqlite".into(),
        version: "0.31.0".into(),
    }));
}

#[test]
fn test_parse_requirements_txt() {
    let dir = tempdir().unwrap();
    let reqs = "requests==2.31.0\n# Comentario\nfastapi==0.110.0\n";
    std::fs::write(dir.path().join("requirements.txt"), reqs).unwrap();

    let deps = parse(&dir.path().to_string_lossy());
    assert_eq!(deps.len(), 2);
    assert_eq!(deps[0].ecosystem, "PyPI");
    assert_eq!(deps[0].name, "requests");
    assert_eq!(deps[0].version, "2.31.0");
    assert_eq!(deps[1].name, "fastapi");
    assert_eq!(deps[1].version, "0.110.0");
}

#[test]
fn test_parse_package_lock_json() {
    let dir = tempdir().unwrap();
    let pkg_lock = r#"{
      "name": "colmeia-ui",
      "version": "1.0.0",
      "packages": {
        "": { "name": "colmeia-ui", "version": "1.0.0" },
        "node_modules/react": { "version": "18.2.0" }
      }
    }"#;
    std::fs::write(dir.path().join("package-lock.json"), pkg_lock).unwrap();

    let deps = parse(&dir.path().to_string_lossy());
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].ecosystem, "npm");
    assert_eq!(deps[0].name, "react");
    assert_eq!(deps[0].version, "18.2.0");
}
