use colmeia::gate::blob_sha;
use colmeia::slice::partition_units;

#[test]
fn test_rfc0001_deterministic_recipe_hashes() {
    let units = vec![
        ("src/z_engine.rs".to_string(), 120),
        ("src/a_core.rs".to_string(), 300),
        ("src/m_mid.rs".to_string(), 90),
    ];

    let plan1 = partition_units(&units, 200);
    let plan2 = partition_units(&units, 200);

    let json1 = serde_json::to_string(&plan1).unwrap();
    let json2 = serde_json::to_string(&plan2).unwrap();

    let hash1 = blob_sha(&json1);
    let hash2 = blob_sha(&json2);

    // Contrato estrito: bit-idêntico em todas as invocações
    assert_eq!(json1, json2);
    assert_eq!(hash1, hash2);
}
