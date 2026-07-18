//! Livello desktop di Oops: espone al frontend i comandi richiamabili via
//! `invoke` e fa da ponte verso il crate `oops_core` (la logica Git).
//!
//! Quasi tutti i comandi ricevono il `percorso` del repository: il core apre
//! il repository ogni volta (è economico) e svolge l'operazione. L'unico stato
//! che teniamo è il file dei "repository recenti".

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use notify::{RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};

use oops_core::model::{
    Calore, ConfigUtente, ConflittoVersioni, Credenziali, FileModificato, Insights, MossaRebase,
    Panoramica, Ramo, Remoto, RepoRecente, StatFile, StatoRepo, Submodulo, Tag, VoceBlame, VoceLog,
    VoceReflog, VoceStash, VoceWorktree, Workspace,
};
use oops_core::{
    azioni, blame, commit, conflitti, contenuto, diff, esegui, insights, panoramica, patch, rami,
    rebase_int, reflog, remote, repo, stage, stash, storage, submoduli, tag, worktree,
};
use tauri::{Manager, State};

/// Percorso del file JSON con i repository recenti (in app_config_dir).
struct FileRecenti(PathBuf);

/// Percorso del file JSON con i workspace (gruppi di repository).
struct FileWorkspaces(PathBuf);

/// L'osservatore del filesystem del repository aperto (per l'auto-refresh).
#[derive(Default)]
struct Osservatore(Mutex<Option<notify::RecommendedWatcher>>);

/// Avvia (o sostituisce) l'osservatore sulla cartella del repository: a ogni
/// cambiamento emette l'evento `oops-fs`, che la UI usa per ricaricare.
#[tauri::command]
fn avvia_osservatore(
    percorso: String,
    app: AppHandle,
    osservatore: State<Osservatore>,
) -> Result<(), String> {
    let app2 = app.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if res.is_ok() {
            let _ = app2.emit("oops-fs", ());
        }
    })
    .map_err(|e| e.to_string())?;

    watcher
        .watch(Path::new(&percorso), RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    // Sostituisce l'eventuale osservatore precedente (che si ferma).
    *osservatore.0.lock().unwrap() = Some(watcher);
    Ok(())
}

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
fn clona(
    url: String,
    destinazione: String,
    cred: Option<Credenziali>,
    insicuro: bool,
) -> Result<String, String> {
    repo::clona(&url, &destinazione, cred, insicuro)
}

// ---- Stato e cronologia ----

#[tauri::command]
fn stato(percorso: String) -> Result<StatoRepo, String> {
    repo::stato(&percorso)
}

#[tauri::command]
fn panoramica(percorso: String) -> Result<Panoramica, String> {
    panoramica::panoramica(&percorso)
}

#[tauri::command]
fn log(percorso: String, limite: usize) -> Result<Vec<VoceLog>, String> {
    commit::log(&percorso, limite)
}

#[tauri::command]
fn log_file(percorso: String, file: String, limite: usize) -> Result<Vec<VoceLog>, String> {
    commit::log_file(&percorso, &file, limite)
}

#[tauri::command]
fn blame_file(percorso: String, file: String) -> Result<Vec<VoceBlame>, String> {
    blame::blame(&percorso, &file)
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

#[tauri::command]
fn scarta_tutto(percorso: String) -> Result<(), String> {
    stage::scarta_tutto(&percorso)
}

#[tauri::command]
fn pulisci_non_tracciati(percorso: String) -> Result<(), String> {
    stage::pulisci_non_tracciati(&percorso)
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

#[tauri::command]
fn condensa(percorso: String, id: String, messaggio: String) -> Result<String, String> {
    commit::condensa(&percorso, &id, &messaggio)
}

#[tauri::command]
fn ripristina_file(percorso: String, id: String, file: String) -> Result<(), String> {
    azioni::ripristina_file(&percorso, &id, &file)
}

// ---- Diff ----

#[tauri::command]
fn diff_file(
    percorso: String,
    file: String,
    in_stage: bool,
    ignora_spazi: bool,
) -> Result<String, String> {
    diff::file(&percorso, &file, in_stage, ignora_spazi)
}

#[tauri::command]
fn diff_commit(percorso: String, id: String, ignora_spazi: bool) -> Result<String, String> {
    diff::commit(&percorso, &id, ignora_spazi)
}

#[tauri::command]
fn diff_commit_genitore(
    percorso: String,
    id: String,
    genitore: usize,
    ignora_spazi: bool,
) -> Result<String, String> {
    diff::commit_vs_genitore(&percorso, &id, genitore, ignora_spazi)
}

#[tauri::command]
fn lista_file_commit(percorso: String, id: String) -> Result<Vec<FileModificato>, String> {
    diff::lista_file_commit(&percorso, &id)
}

#[tauri::command]
fn diff_commit_file(
    percorso: String,
    id: String,
    file: String,
    ignora_spazi: bool,
) -> Result<String, String> {
    diff::commit_file(&percorso, &id, &file, ignora_spazi)
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

#[tauri::command]
fn stash_diff(percorso: String, indice: usize) -> Result<String, String> {
    stash::diff(&percorso, indice)
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
fn cherry_pick_su(percorso: String, id: String, ramo: String) -> Result<(), String> {
    azioni::cherry_pick_su(&percorso, &id, &ramo)
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

#[tauri::command]
fn ramo_rebase(percorso: String, su: String) -> Result<String, String> {
    rami::rebase(&percorso, &su)
}

#[tauri::command]
fn ramo_crea_da(percorso: String, nome: String, id: String, cambia: bool) -> Result<(), String> {
    rami::crea_da(&percorso, &nome, &id, cambia)
}

#[tauri::command]
fn ramo_checkout_commit(percorso: String, id: String) -> Result<(), String> {
    rami::checkout_commit(&percorso, &id)
}

// ---- Revert ----

#[tauri::command]
fn revert(percorso: String, id: String) -> Result<(), String> {
    azioni::revert(&percorso, &id)
}

// ---- Conflitti ----

#[tauri::command]
fn conflitti_lista(percorso: String) -> Result<Vec<String>, String> {
    conflitti::lista(&percorso)
}

#[tauri::command]
fn conflitto_risolvi(percorso: String, file: String, lato: String) -> Result<(), String> {
    conflitti::risolvi(&percorso, &file, &lato)
}

#[tauri::command]
fn conflitto_segna_risolto(percorso: String, file: String) -> Result<(), String> {
    conflitti::segna_risolto(&percorso, &file)
}

#[tauri::command]
fn operazione_annulla(percorso: String) -> Result<(), String> {
    conflitti::annulla(&percorso)
}

#[tauri::command]
fn conflitto_versioni(percorso: String, file: String) -> Result<ConflittoVersioni, String> {
    conflitti::versioni(&percorso, &file)
}

#[tauri::command]
fn conflitto_salva(percorso: String, file: String, contenuto: String) -> Result<(), String> {
    conflitti::salva(&percorso, &file, &contenuto)
}

// ---- Rebase interattivo ----

#[tauri::command]
fn rebase_interattivo(
    percorso: String,
    base: String,
    mosse: Vec<MossaRebase>,
) -> Result<String, String> {
    rebase_int::esegui(&percorso, &base, mosse)
}

// ---- Strumenti avanzati: reflog, submoduli, worktree, patch ----

#[tauri::command]
fn reflog_lista(percorso: String) -> Result<Vec<VoceReflog>, String> {
    reflog::lista(&percorso)
}

#[tauri::command]
fn submoduli_lista(percorso: String) -> Result<Vec<Submodulo>, String> {
    submoduli::lista(&percorso)
}

#[tauri::command]
fn submodulo_aggiorna(percorso: String, nome: String) -> Result<(), String> {
    submoduli::aggiorna(&percorso, &nome)
}

#[tauri::command]
fn worktree_lista(percorso: String) -> Result<Vec<VoceWorktree>, String> {
    worktree::lista(&percorso)
}

#[tauri::command]
fn worktree_aggiungi(percorso: String, nome: String, cartella: String) -> Result<(), String> {
    worktree::aggiungi(&percorso, &nome, &cartella)
}

#[tauri::command]
fn patch_esporta(percorso: String, id: String, destinazione: String) -> Result<(), String> {
    patch::esporta(&percorso, &id, &destinazione)
}

#[tauri::command]
fn patch_applica(percorso: String, file: String) -> Result<(), String> {
    patch::applica(&percorso, &file)
}

// ---- Remoti ----

#[tauri::command]
fn remoti_lista(percorso: String) -> Result<Vec<String>, String> {
    remote::lista(&percorso)
}

#[tauri::command]
fn fetch(
    percorso: String,
    remoto: String,
    cred: Option<Credenziali>,
    insicuro: bool,
) -> Result<(), String> {
    remote::fetch(&percorso, &remoto, cred, insicuro)
}

#[tauri::command]
fn pull(
    percorso: String,
    remoto: String,
    strategia: String,
    cred: Option<Credenziali>,
    insicuro: bool,
) -> Result<String, String> {
    remote::pull(&percorso, &remoto, &strategia, cred, insicuro)
}

#[tauri::command]
fn push(
    percorso: String,
    remoto: String,
    forza: bool,
    cred: Option<Credenziali>,
    insicuro: bool,
) -> Result<(), String> {
    remote::push(&percorso, &remoto, forza, cred, insicuro)
}

#[tauri::command]
fn push_tags(
    percorso: String,
    remoto: String,
    cred: Option<Credenziali>,
    insicuro: bool,
) -> Result<(), String> {
    remote::push_tags(&percorso, &remoto, cred, insicuro)
}

#[tauri::command]
fn remoti_dettagli(percorso: String) -> Result<Vec<Remoto>, String> {
    remote::lista_dettagli(&percorso)
}

#[tauri::command]
fn remoto_aggiungi(percorso: String, nome: String, url: String) -> Result<(), String> {
    remote::aggiungi(&percorso, &nome, &url)
}

#[tauri::command]
fn remoto_imposta_url(percorso: String, nome: String, url: String) -> Result<(), String> {
    remote::imposta_url(&percorso, &nome, &url)
}

#[tauri::command]
fn remoto_rimuovi(percorso: String, nome: String) -> Result<(), String> {
    remote::rimuovi(&percorso, &nome)
}

#[tauri::command]
fn elimina_ramo_remoto(
    percorso: String,
    remoto: String,
    ramo: String,
    cred: Option<Credenziali>,
    insicuro: bool,
) -> Result<(), String> {
    remote::elimina_ramo_remoto(&percorso, &remoto, &ramo, cred, insicuro)
}

// ---- Drag&drop ramo-su-ramo, statistiche, insights, undo, terminale ----

#[tauri::command]
fn merge_rami(percorso: String, sorgente: String, destinazione: String) -> Result<String, String> {
    rami::merge_rami(&percorso, &sorgente, &destinazione)
}

#[tauri::command]
fn rebase_rami(percorso: String, sorgente: String, destinazione: String) -> Result<String, String> {
    rami::rebase_rami(&percorso, &sorgente, &destinazione)
}

#[tauri::command]
fn stat_lavoro(percorso: String, in_stage: bool) -> Result<Vec<StatFile>, String> {
    diff::stat_lavoro(&percorso, in_stage)
}

#[tauri::command]
fn stat_commit(percorso: String, id: String) -> Result<Vec<StatFile>, String> {
    diff::stat_commit(&percorso, &id)
}

#[tauri::command]
fn calore(percorso: String, limite: usize) -> Result<Vec<Calore>, String> {
    diff::calore(&percorso, limite)
}

#[tauri::command]
fn insights(percorso: String, limite: usize) -> Result<Insights, String> {
    insights::insights(&percorso, limite)
}

#[tauri::command]
fn annulla_ultima(percorso: String) -> Result<String, String> {
    azioni::annulla_ultima(&percorso)
}

#[tauri::command]
fn esegui_git(percorso: String, args: Vec<String>) -> Result<String, String> {
    esegui::git(&percorso, args)
}

// ---- Compare, stage per riga, cherry-pick avanzato, release notes ----

#[tauri::command]
fn diff_tra_commit(percorso: String, a: String, b: String, ignora_spazi: bool) -> Result<String, String> {
    diff::tra_commit(&percorso, &a, &b, ignora_spazi)
}

#[tauri::command]
fn stage_righe(percorso: String, patch: String, reverse: bool) -> Result<(), String> {
    diff::applica_indice(&percorso, &patch, reverse)
}

#[tauri::command]
fn cherry_pick_squash(percorso: String, id: String, ramo: String) -> Result<(), String> {
    azioni::cherry_pick_squash(&percorso, &id, &ramo)
}

#[tauri::command]
fn cherry_pick_muovi(percorso: String, id: String, ramo: String) -> Result<(), String> {
    azioni::cherry_pick_muovi(&percorso, &id, &ramo)
}

#[tauri::command]
fn commit_rimuovi(percorso: String, id: String) -> Result<String, String> {
    rebase_int::rimuovi(&percorso, &id)
}

#[tauri::command]
fn note_release(percorso: String, da: String, a: String) -> Result<Vec<VoceLog>, String> {
    commit::tra(&percorso, &da, &a, 500)
}

// ---- Workspace ----

#[tauri::command]
fn workspace_lista(file: State<FileWorkspaces>) -> Vec<Workspace> {
    storage::workspace_carica(&file.0)
}

#[tauri::command]
fn workspace_salva(
    nome: String,
    percorsi: Vec<String>,
    file: State<FileWorkspaces>,
) -> Result<Vec<Workspace>, String> {
    storage::workspace_salva(&file.0, &nome, percorsi)
}

#[tauri::command]
fn workspace_elimina(nome: String, file: State<FileWorkspaces>) -> Result<Vec<Workspace>, String> {
    storage::workspace_elimina(&file.0, &nome)
}

// ---- Anteprima file (markdown / immagini) ----

#[tauri::command]
fn leggi_testo_lavoro(percorso: String, file: String) -> Result<String, String> {
    contenuto::testo_lavoro(&percorso, &file)
}

#[tauri::command]
fn leggi_b64_lavoro(percorso: String, file: String) -> Result<String, String> {
    contenuto::b64_lavoro(&percorso, &file)
}

#[tauri::command]
fn leggi_b64_head(percorso: String, file: String) -> Result<String, String> {
    contenuto::b64_head(&percorso, &file)
}

// ---- Assistente AI (Anthropic): messaggio di commit dal diff in stage ----

#[tauri::command]
fn genera_commit_ai(percorso: String, token: String, modello: String) -> Result<String, String> {
    if token.trim().is_empty() {
        return Err("manca il token API: impostalo nelle Impostazioni".into());
    }
    let diff = diff::staged_tutto(&percorso)?;
    if diff.trim().is_empty() {
        return Err("niente in stage: aggiungi dei file prima di generare il messaggio".into());
    }
    // Tronca i diff enormi per non sforare i limiti/costi.
    let diff: String = diff.chars().take(12_000).collect();
    let modello = if modello.trim().is_empty() {
        "claude-sonnet-5".to_string()
    } else {
        modello
    };

    let prompt = format!(
        "Sei un assistente che scrive messaggi di commit Git in italiano. Dato il seguente \
diff in stage, scrivi UN SOLO messaggio di commit conciso: una prima riga (max ~70 caratteri) \
in stile convenzionale (es. \"fix: …\", \"feat: …\"), poi se utile una riga vuota e un breve corpo. \
Rispondi SOLO col messaggio, senza virgolette o spiegazioni.\n\nDIFF:\n{diff}"
    );

    let corpo = serde_json::json!({
        "model": modello,
        "max_tokens": 1024,
        "messages": [{ "role": "user", "content": prompt }]
    });

    match ureq::post("https://api.anthropic.com/v1/messages")
        .set("x-api-key", token.trim())
        .set("anthropic-version", "2023-06-01")
        .set("content-type", "application/json")
        .send_json(corpo)
    {
        Ok(r) => {
            let v: serde_json::Value = r.into_json().map_err(|e| e.to_string())?;
            let testo = v["content"][0]["text"].as_str().unwrap_or("").trim().to_string();
            if testo.is_empty() {
                Err("risposta dell'AI vuota".into())
            } else {
                Ok(testo)
            }
        }
        Err(ureq::Error::Status(code, r)) => {
            let dett = r.into_string().unwrap_or_default();
            Err(format!("errore API Anthropic ({code}): {dett}"))
        }
        Err(e) => Err(format!("errore di rete: {e}")),
    }
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
        .manage(Osservatore::default())
        .setup(|app| {
            // I file di config (recenti, workspace) vivono nella cartella dell'app.
            let dir = app.path().app_config_dir()?;
            app.manage(FileRecenti(dir.join("recenti.json")));
            app.manage(FileWorkspaces(dir.join("workspace.json")));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            apri_repo,
            init_repo,
            clona,
            stato,
            panoramica,
            log,
            log_file,
            blame_file,
            stage_aggiungi,
            stage_aggiungi_tutto,
            stage_togli,
            stage_togli_tutto,
            scarta,
            scarta_tutto,
            pulisci_non_tracciati,
            crea_commit,
            amend,
            ultimo_messaggio,
            condensa,
            ripristina_file,
            diff_file,
            diff_commit,
            diff_commit_genitore,
            lista_file_commit,
            diff_commit_file,
            hunk_stage,
            hunk_scarta,
            stash_lista,
            stash_salva,
            stash_applica,
            stash_pop,
            stash_elimina,
            stash_diff,
            tag_lista,
            tag_crea,
            tag_elimina,
            reset_commit,
            cherry_pick,
            cherry_pick_su,
            config_utente,
            imposta_config_utente,
            rami_lista,
            ramo_crea,
            ramo_checkout,
            ramo_elimina,
            ramo_merge,
            ramo_rebase,
            ramo_crea_da,
            ramo_checkout_commit,
            revert,
            conflitti_lista,
            conflitto_risolvi,
            conflitto_segna_risolto,
            operazione_annulla,
            conflitto_versioni,
            conflitto_salva,
            rebase_interattivo,
            reflog_lista,
            submoduli_lista,
            submodulo_aggiorna,
            worktree_lista,
            worktree_aggiungi,
            patch_esporta,
            patch_applica,
            remoti_lista,
            remoti_dettagli,
            remoto_aggiungi,
            remoto_imposta_url,
            remoto_rimuovi,
            elimina_ramo_remoto,
            fetch,
            pull,
            push,
            push_tags,
            recenti_lista,
            recenti_aggiungi,
            recenti_rimuovi,
            avvia_osservatore,
            merge_rami,
            rebase_rami,
            stat_lavoro,
            stat_commit,
            calore,
            insights,
            annulla_ultima,
            esegui_git,
            diff_tra_commit,
            stage_righe,
            cherry_pick_squash,
            cherry_pick_muovi,
            commit_rimuovi,
            note_release,
            workspace_lista,
            workspace_salva,
            workspace_elimina,
            leggi_testo_lavoro,
            leggi_b64_lavoro,
            leggi_b64_head,
            genera_commit_ai,
        ])
        .run(tauri::generate_context!())
        .expect("errore durante l'avvio di Oops");
}
