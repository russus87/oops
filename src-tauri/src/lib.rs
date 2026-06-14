//! Livello desktop di Oops: espone al frontend i comandi richiamabili via
//! `invoke` e fa da ponte verso il crate `oops_core` (la logica Git).
//!
//! Quasi tutti i comandi ricevono il `percorso` del repository: il core apre
//! il repository ogni volta (è economico) e svolge l'operazione. L'unico stato
//! che teniamo è il file dei "repository recenti".

use std::path::PathBuf;

use oops_core::model::{Ramo, RepoRecente, StatoRepo, VoceLog};
use oops_core::{commit, diff, rami, remote, repo, stage, storage};
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

// ---- Diff ----

#[tauri::command]
fn diff_file(percorso: String, file: String, in_stage: bool) -> Result<String, String> {
    diff::file(&percorso, &file, in_stage)
}

#[tauri::command]
fn diff_commit(percorso: String, id: String) -> Result<String, String> {
    diff::commit(&percorso, &id)
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
            diff_file,
            diff_commit,
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
