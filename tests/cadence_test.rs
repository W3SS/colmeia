//! F03 — Cron do scheduler: política dispara uma única vez por ocorrência.

use colmeia::cadence;
use colmeia::store::Store;

#[test]
fn is_due_janela_e_last_run() {
    let agora = colmeia::store::unix();
    // cron a cada segundo: disparou há 30 s → ainda vencido se não executado
    assert!(cadence::is_due("* * * * * *", None));
    // já executado após o disparo → não repete
    assert!(!cadence::is_due("* * * * * *", Some(agora)));
    // execução antiga (antes da janela) → vencido de novo
    assert!(cadence::is_due("* * * * * *", Some(agora - 300)));
    // expressão inválida nunca dispara
    assert!(!cadence::is_due("não é cron", None));
}

#[test]
fn tick_dispara_uma_vez() {
    let store = Store::open_in_memory().unwrap();
    let repo = store.add_repo("/tmp/inexistente-path-sync", None, "mirror").unwrap();
    store
        .add_policy("fetch_upstream", Some(repo), None, Some("origin"), None, Some("* * * * * *"))
        .unwrap();

    let fired1 = cadence::tick(&store).unwrap();
    assert_eq!(fired1, 1, "primeiro tick deve disparar a política");

    let fired2 = cadence::tick(&store).unwrap();
    assert_eq!(fired2, 0, "segundo tick imediato não deve repetir (last_run)");

    // O job foi enfileirado exatamente uma vez
    let jobs = store.recent_jobs(10).unwrap();
    assert_eq!(jobs.iter().filter(|j| j.1 == "fetch").count(), 1);
}

#[test]
fn tick_politica_sem_cron_ignora() {
    let store = Store::open_in_memory().unwrap();
    let repo = store.add_repo("/tmp/inexistente-path-sync-2", None, "mirror").unwrap();
    store
        .add_policy("fetch_upstream", Some(repo), None, None, None, None)
        .unwrap();
    assert_eq!(cadence::tick(&store).unwrap(), 0);
}
