use colmeia::config::Config;
use colmeia::governor::{Governor, MemoryPressure};

#[test]
fn test_governor_thresholds() {
    let cfg = Config {
        memory_soft_mb: 100,
        memory_hard_mb: 200,
        ..Config::default()
    };

    let gov = Governor::new(&cfg);
    assert_eq!(gov.soft_mb, 100);
    assert_eq!(gov.hard_mb, 200);

    // Em ambiente de teste normal com consumo baixo, a pressão deve ser Normal
    let pressure = gov.check();
    assert!(matches!(
        pressure,
        MemoryPressure::Normal | MemoryPressure::SoftLimit
    ));
}

#[test]
fn test_detect_tier() {
    let tier = Config::detect_tier();
    assert!(tier == "t1" || tier == "t2" || tier == "t3");
}
