//! F54 — chaos: kill -9 ⇒ recovery. Lease expirado é re-aquisitável e
//! dependências do DAG respeitam a ordem.

use colmeia::store::Store;

#[test]
fn lease_expirado_e_readquirido_apos_crash() {
    let store = Store::open_in_memory().unwrap();
    let job = store
        .enqueue("fetch", 1, &serde_json::json!({}), 5)
        .unwrap();

    // Worker pega o job e "morre" (não chama finish)
    let (id1, kind1, _, _) = store.lease_next(1).unwrap().unwrap();
    assert_eq!(id1, job);
    assert_eq!(kind1, "fetch");

    // TTL de 1 s: espera expirar e re-aquira (simula recovery pós kill -9)
    std::thread::sleep(std::time::Duration::from_millis(1200));
    let leased = store.lease_next(1).unwrap().unwrap();
    assert_eq!(leased.0, job, "job com lease expirado deve ser re-aquisitável");

    // Conclui: não volta para a fila
    store.finish(job, None).unwrap();
    assert!(store.lease_next(1).unwrap().is_none());
}

#[test]
fn dag_dependencia_respeita_ordem() {
    let store = Store::open_in_memory().unwrap();
    let a = store.enqueue("fetch", 1, &serde_json::json!({}), 5).unwrap();
    let _b = store
        .enqueue_dependent("gate", 1, &serde_json::json!({}), 5, Some(a))
        .unwrap();

    // b tem dependência pendente → só a pode ser leased
    let first = store.lease_next(600).unwrap().unwrap();
    assert_eq!(first.0, a);

    // a ainda running (não done) → nada mais disponível
    assert!(store.lease_next(600).unwrap().is_none());

    // a conclui → b aparece
    store.finish(a, None).unwrap();
    let second = store.lease_next(600).unwrap().unwrap();
    assert_eq!(second.1, "gate");
}

#[test]
fn falha_marca_job_e_error_persiste() {
    let store = Store::open_in_memory().unwrap();
    let job = store.enqueue("fetch", 1, &serde_json::json!({}), 5).unwrap();
    store.lease_next(600).unwrap();
    store.finish(job, Some("rede caiu")).unwrap();

    let jobs = store.recent_jobs(5).unwrap();
    let rec = jobs.iter().find(|j| j.0 == job).unwrap();
    assert_eq!(rec.3, "failed");
}
