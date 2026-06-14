//! Livello desktop di Oops: espone al frontend i comandi richiamabili via
//! `invoke` e fa da ponte verso il crate `oops_core` (la logica Git).
//!
//! Quasi tutti i comandi ricevono il `percorso` del repository: il core apre
//! il repository ogni volta (è economico) e svolge l'operazione. L'unico stato
//! che teniamo è il file dei "repository recenti".

use std::path::PathBuf;

use oops_core::model::{
    ConfigUtente, FileModificato, Ramo, RepoRecente, StatoRepo, Tag, VoceLog, VoceStash,
};
use oops_core::{azioni, commit, diff, rami, remote, repo, stage, stash, storage, tag};
use tauri::{Manager, State};

/// Percorso del file JSON con i repository recenti (in app_config_dir).
struct FileRecenti(PathBuf);

// ---- Apertura repository ----

#[tauri::command]
fn apri_repo(percorso: String) -> Result<String, String> {
    repo::apri_repo(&percorso)
}

#[tauri::command]
fn init_repo(percorso: String) -> Result<String, String> {
    repo::init(&percorso)
}

#[tauri::command]
fn clona(url: String, destinazione: String) -> Result<String, String> {
    repo::clona(&url, &destinazione)
}

// ---- Stato e cronologia ----

#[tauri::command]
fn stato(percorso: String) -> Result<StatoRepo, String> {
    repo::stato(&percorso)
}

#[tauri::command]
fn log(percorso: String, limite: usize) -> Result<Vec<VoceLog>, String> {
    commit::log(&percorso, limite)
}

// ---- Staging ----

#[tauri::command]
fn stage_aggiungi(percorso: String, file: String) -> Result<(), String> {
    stage::aggiungi(&percorso, &file)
}

#[tauri::command]
fn stage_aggiungi_tutto(percorso: String) -> Result<(), String> {
    stage::aggiungi_tutto(&percorso)
}

#[tauri::command]
fn stage_togli(percorso: String, file: String) -> Result<(), String> {
    stage::togli(&percorso, &file)
}

#[tauri::command]
fn stage_togli_tutto(percorso: String) -> Result<(), String> {
    stage::togli_tutto(&percorso)
}

#[tauri::command]
fn scarta(percorso: String, file: String) -> Result<(), String> {
    stage::scarta(&percorso, &file)
}

// ---- Commit ----

#[tauri::command]
fn crea_commit(
    percorso: String,
    messaggio: String,
    nome: String,
    email: String,
) -> Result<String, String> {
    commit::crea(&percorso, &messaggio, &nome, &email)
}

// ---- Commit: amend ----

#[tauri::command]
fn amend(percorso: String, messaggio: String) -> Result<String, String> {
    commit::amend(&percorso, &messaggio)
}

#[tauri::command]
fn ultimo_messaggio(percorso: String) -> Result<String, String> {
    commit::ultimo_messaggio(&percorso)
}

// ---- Diff ----

#[tauri::command]
fn diff_file(percorso: String, file: String, in_stage: bool) -> Result<String, String> {
    diff::file(&percorso, &file, in_stage)
}

#[tauri::command]
fn diff_commit(percorso: String, id: String) -> Result<String, String> {
    diff::commit(&percorso, &id)
}

#[tauri::command]
fn lista_file_commit(percorso: String, id: String) -> Result<Vec<FileModificato>, String> {
    diff::lista_file_commit(&percorso, &id)
}

#[tauri::command]
fn diff_commit_file(percorso: String, id: String, file: String) -> Result<String, String> {
    diff::commit_file(&percorso, &id, &file)
}

// ---- Diff per hunk ----

#[tauri::command]
fn hunk_stage(
    percorso: String,
    file: String,
    indice: usize,
    in_stage: bool,
) -> Result<(), String> {
    diff::hunk_stage(&percorso, &file, indice, in_stage)
}

#[tauri::command]
fn hunk_scarta(percorso: String, file: String, indice: usize) -> Result<(), String> {
    diff::hunk_scarta(&percorso, &file, indice)
}

// ---- Stash ----

#[tauri::command]
fn stash_lista(percorso: String) -> Result<Vec<VoceStash>, String> {
    stash::lista(&percorso)
}

#[tauri::command]
fn stash_salva(
    percorso: String,
    messaggio: String,
    includi_non_tracciati: bool,
) -> Result<(), String> {
    stash::salva(&percorso, &messaggio, includi_non_tracciati)
}

#[tauri::command]
fn stash_applica(percorso: String, indice: usize) -> Result<(), String> {
    stash::applica(&percorso, indice)
}

#[tauri::command]
fn stash_pop(percorso: String, indice: usize) -> Result<(), String> {
    stash::pop(&percorso, indice)
}

#[tauri::command]
fn stash_elimina(percorso: String, indice: usize) -> Result<(), String> {
    stash::elimina(&percorso, indice)
}

// ---- Tag ----

#[tauri::command]
fn tag_lista(percorso: String) -> Result<Vec<Tag>, String> {
    tag::lista(&percorso)
}

#[tauri::command]
fn tag_crea(percorso: String, nome: String, messaggio: String) -> Result<(), String> {
    tag::crea(&percorso, &nome, &messaggio)
}

#[tauri::command]
fn tag_elimina(percorso: String, nome: String) -> Result<(), String> {
    tag::elimina(&percorso, &nome)
}

// ---- Azioni su commit (reset, cherry-pick, config autore) ----

#[tauri::command]
fn reset_commit(percorso: String, id: String, modo: String) -> Result<(), String> {
    azioni::reset(&percorso, &id, &modo)
}

#[tauri::command]
fn cherry_pick(percorso: String, id: String) -> Result<(), String> {
    azioni::cherry_pick(&percorso, &id)
}

#[tauri::command]
fn config_utente(percorso: String) -> Result<ConfigUtente, String> {
    azioni::config_utente(&percorso)
}

#[tauri::command]
fn imposta_config_utente(percorso: String, nome: String, email: String) -> Result<(), String> {
    azioni::imposta_config_utente(&percorso, &nome, &email)
}

// ---- Rami ----

#[tauri::command]
fn rami_lista(percorso: String) -> Result<Vec<Ramo>, String> {
    rami::lista(&percorso)
}

#[tauri::command]
fn ramo_crea(percorso: String, nome: String, cambia: bool) -> Result<(), String> {
    rami::crea(&percorso, &nome, cambia)
}

#[tauri::command]
fn ramo_checkout(percorso: String, nome: String) -> Result<(), String> {
    rami::checkout(&percorso, &nome)
}

#[tauri::command]
fn ramo_elimina(percorso: String, nome: String) -> Result<(), String> {
    rami::elimina(&percorso, &nome)
}

#[tauri::command]
fn ramo_merge(percorso: String, nome: String) -> Result<String, String> {
    rami::merge(&percorso, &nome)
}

// ---- Remoti ----

#[tauri::command]
fn remoti_lista(percorso: String) -> Result<Vec<String>, String> {
    remote::lista(&percorso)
}

#[tauri::command]
fn fetch(percorso: String, remoto: String) -> Result<(), String> {
    remote::fetch(&percorso, &remoto)
}

#[tauri::command]
fn pull(percorso: String, remoto: String) -> Result<String, String> {
    remote::pull(&percorso, &remoto)
}

#[tauri::command]
fn push(percorso: String, remoto: String) -> Result<(), String> {
    remote::push(&percorso, &remoto)
}

// ---- Repository recenti ----

#[tauri::command]
fn recenti_lista(file: State<FileRecenti>) -> Vec<RepoRecente> {
    storage::carica(&file.0)
}

#[tauri::command]
fn recenti_aggiungi(
    percorso: String,
    file: State<FileRecenti>,
) -> Result<Vec<RepoRecente>, String> {
    storage::aggiungi(&file.0, &percorso)
}

#[tauri::command]
fn recenti_rimuovi(
    percorso: String,
    file: State<FileRecenti>,
) -> Result<Vec<RepoRecente>, String> {
    storage::rimuovi(&file.0, &percorso)
}

/// Punto di ingresso dell'app desktop.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Il file dei recenti vive nella cartella di config dell'app.
            let dir = app.path().app_config_dir()?;
            app.manage(FileRecenti(dir.join("recenti.json")));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            apri_repo,
            init_repo,
            clona,
            stato,
            log,
            stage_aggiungi,
            stage_aggiungi_tutto,
            stage_togli,
            stage_togli_tutto,
            scarta,
            crea_commit,
            amend,
            ultimo_messaggio,
            diff_file,
            diff_commit,
            lista_file_commit,
            diff_commit_file,
            hunk_stage,
            hunk_scarta,
            stash_lista,
            stash_salva,
            stash_applica,
            stash_pop,
            stash_elimina,
            tag_lista,
            tag_crea,
            tag_elimina,
            reset_commit,
            cherry_pick,
            config_utente,
            imposta_config_utente,
            rami_lista,
            ramo_crea,
            ramo_checkout,
            ramo_elimina,
            ramo_merge,
            remoti_lista,
            fetch,
            pull,
            push,
            recenti_lista,
            recenti_aggiungi,
            recenti_rimuovi,
        ])
        .run(tauri::generate_context!())
        .expect("errore durante l'avvio di Oops");
}
