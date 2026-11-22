use colmeia::slice::partition_units;

#[test]
fn test_partition_units_respects_max_lines() {
    let units = vec![
        ("src/a.rs".to_string(), 150),
        ("src/b.rs".to_string(), 200),
        ("src/c.rs".to_string(), 180),
        ("src/d.rs".to_string(), 90),
    ];

    // Com max_lines = 300:
    // a.rs (150) + b.rs (200) = 350 > 300 => a.rs fica no slice 1
    // b.rs (200) + c.rs (180) = 380 > 300 => b.rs fica no slice 2
    // c.rs (180) + d.rs (90) = 270 <= 300 => c.rs e d.rs ficam no slice 3
    let slices = partition_units(&units, 300);

    assert_eq!(slices.len(), 3);
    assert_eq!(slices[0], vec!["src/a.rs"]);
    assert_eq!(slices[1], vec!["src/b.rs"]);
    assert_eq!(slices[2], vec!["src/c.rs", "src/d.rs"]);
}

#[test]
fn test_partition_units_deterministic_sorting() {
    // Ordem de entrada desordenada
    let units1 = vec![
        ("z.rs".to_string(), 50),
        ("a.rs".to_string(), 50),
        ("m.rs".to_string(), 50),
    ];

    let units2 = vec![
        ("m.rs".to_string(), 50),
        ("z.rs".to_string(), 50),
        ("a.rs".to_string(), 50),
    ];

    let slices1 = partition_units(&units1, 1000);
    let slices2 = partition_units(&units2, 1000);

    // Ambas devem ser estritamente idênticas
    assert_eq!(slices1, slices2);
    assert_eq!(slices1[0], vec!["a.rs", "m.rs", "z.rs"]);
}
