use colmeia::store::Store;

#[test]
fn test_store_open_and_repo_management() {
    let store = Store::open_in_memory().expect("Falha ao abrir store em memória");

    let id1 = store
        .add_repo(
            "/tmp/repo1",
            Some("https://github.com/org/repo1.git"),
            "managed",
        )
        .expect("Falha ao adicionar repo1");
    assert_eq!(id1, 1);

    // Adição idempotente com INSERT OR IGNORE
    let id1_repeat = store
        .add_repo("/tmp/repo1", None, "managed")
        .expect("Falha ao adicionar repo1 repetido");
    assert_eq!(id1_repeat, 1);

    let id2 = store
        .add_repo("/tmp/repo2", None, "mirror")
        .expect("Falha ao adicionar repo2");
    assert_eq!(id2, 2);

    let repos = store.list_repos().expect("Falha ao listar repos");
    assert_eq!(repos.len(), 2);
    assert_eq!(repos[0].1, "/tmp/repo1");
    assert_eq!(repos[1].1, "/tmp/repo2");
}

#[test]
fn test_jobs_queue_lease_and_crash_recovery() {
    let store = Store::open_in_memory().expect("Falha ao abrir store em memória");
    let repo_id = store.add_repo("/tmp/repo1", None, "managed").unwrap();

    let job_id = store
        .enqueue(
            "fetch",
            repo_id,
            &serde_json::json!({ "remote": "origin" }),
            5,
        )
        .expect("Falha ao enfileirar job");

    assert_eq!(job_id, 1);

    // 1. Worker aluga o job com lease de 10 segundos
    let leased = store.lease_next(10).expect("Falha ao alugar job");
    assert!(leased.is_some());
    let (id, kind, r_id, payload) = leased.unwrap();
    assert_eq!(id, 1);
    assert_eq!(kind, "fetch");
    assert_eq!(r_id, repo_id);
    assert!(payload.contains("origin"));

    // 2. Enquanto o lease estiver ativo, outro worker não deve conseguir alugar o mesmo job
    let no_lease = store.lease_next(10).expect("Falha na checagem de lease");
    assert!(no_lease.is_none());

    // 3. Simula crash do worker (lease expira definindo lease_until no passado)
    {
        let conn = store.conn.lock().unwrap();
        conn.execute("UPDATE jobs SET lease_until = 0 WHERE id = 1", [])
            .unwrap();
    }

    // 4. Job deve ser re-alugado automaticamente (Crash Recovery)
    let recovered = store.lease_next(10).expect("Falha na recuperação de lease");
    assert!(recovered.is_some());
    assert_eq!(recovered.unwrap().0, 1);

    // 5. Finaliza com sucesso
    store.finish(1, None).expect("Falha ao finalizar job");

    // 6. Fila agora deve estar vazia
    assert!(store.lease_next(10).unwrap().is_none());
}

#[test]
fn test_recent_jobs_returns_status_and_attempts() {
    let store = Store::open_in_memory().expect("Falha ao abrir store em memória");
    let repo_id = store.add_repo("/tmp/repo1", None, "managed").unwrap();
    store
        .enqueue("fetch", repo_id, &serde_json::json!({"remote":"origin"}), 5)
        .unwrap();
    store.lease_next(30).unwrap();

    let jobs = store.recent_jobs(10).unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].1, "fetch");
    assert_eq!(jobs[0].2, repo_id);
    assert_eq!(jobs[0].3, "running");
    assert_eq!(jobs[0].4, 1);
}

#[test]
fn test_dependent_job_waits_for_parent_completion() {
    let store = Store::open_in_memory().expect("Falha ao abrir store em memória");
    let repo_id = store.add_repo("/tmp/repo1", None, "managed").unwrap();
    let parent = store
        .enqueue("fetch", repo_id, &serde_json::json!({}), 1)
        .unwrap();
    let child = store
        .enqueue_dependent("gate", repo_id, &serde_json::json!({}), 1, Some(parent))
        .unwrap();

    assert_eq!(store.lease_next(30).unwrap().unwrap().0, parent);
    assert!(store.lease_next(30).unwrap().is_none());
    store.finish(parent, None).unwrap();
    assert_eq!(store.lease_next(30).unwrap().unwrap().0, child);
}

#[test]
fn test_events_after_returns_incremental_rows_in_order() {
    let store = Store::open_in_memory().expect("Falha ao abrir store em memória");
    let repo_id = store.add_repo("/tmp/repo1", None, "managed").unwrap();
    let first = store
        .event(repo_id, "fetch", &serde_json::json!({"ok":true}))
        .unwrap();
    let second = store
        .event(repo_id, "gate", &serde_json::json!({"ok":true}))
        .unwrap();

    let rows = store.events_after(first, 10).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, second);
    assert_eq!(rows[0].3, "gate");
}

#[test]
fn test_findings_upsert_and_deduplication() {
    let store = Store::open_in_memory().expect("Falha ao abrir store em memória");
    let repo_id = store.add_repo("/tmp/repo1", None, "managed").unwrap();

    store
        .upsert_finding(
            repo_id,
            "blob_sha_123",
            "secret.token",
            "critical",
            "src/main.rs",
            42,
            "Token exposto",
            "ASVS",
        )
        .expect("Falha ao inserir finding");

    // Inserção repetida deve atualizar last_seen sem duplicar linha
    store
        .upsert_finding(
            repo_id,
            "blob_sha_123",
            "secret.token",
            "critical",
            "src/main.rs",
            42,
            "Token exposto",
            "ASVS",
        )
        .expect("Falha no upsert repetido");

    let findings = store.list_findings(repo_id).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].0, "secret.token");
    assert_eq!(findings[0].1, "critical");
    assert_eq!(findings[0].3, 42);
}

#[test]
fn test_recipes_persistence() {
    let store = Store::open_in_memory().expect("Falha ao abrir store em memória");
    let repo_id = store.add_repo("/tmp/repo1", None, "managed").unwrap();

    let recipe_id = store
        .add_recipe(
            repo_id,
            "slice",
            "commit_sha_abc",
            "policy_sha_def",
            "output_sha_ghi",
            r#"{"slices":[["a.rs"],["b.rs"]]}"#,
        )
        .expect("Falha ao persistir receita");

    assert_eq!(recipe_id, 1);
}
