use colmeia::{scanner, store::Store};
use std::process::Command;

#[test]
fn maps_nested_repositories_without_touching_sources() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let first = root.join("group/first");
    let inside_first = first.join("nested-repository");
    let second = root.join("group/nested/second");
    let ignored = root.join("node_modules/hidden");
    for path in [&first, &inside_first, &second, &ignored] {
        std::fs::create_dir_all(path).unwrap();
        let status = Command::new("git")
            .args(["init", "-q"])
            .arg(path)
            .status()
            .unwrap();
        assert!(status.success());
    }

    let store = Store::open_in_memory().unwrap();
    let report = scanner::scan(&store, root.to_str().unwrap()).unwrap();
    assert_eq!(report.discovered, 3);
    assert!(!report.truncated);
    assert_eq!(store.list_scan_roots().unwrap().len(), 1);
    assert_eq!(store.list_repos().unwrap().len(), 3);
    assert!(first.join(".git").is_dir());
    assert!(inside_first.join(".git").is_dir());
    assert!(second.join(".git").is_dir());
    assert!(ignored.join(".git").is_dir());

    scanner::scan(&store, root.to_str().unwrap()).unwrap();
    assert_eq!(store.list_repos().unwrap().len(), 3);
}
