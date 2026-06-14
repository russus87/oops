//! Apertura/clonazione del repository e lettura dello stato dei file
//! (l'equivalente di `git status`).

use std::path::Path;

use git2::{BranchType, Repository, Status, StatusOptions};

use crate::model::{FileModificato, StatoFile, StatoRepo};

/// Verifica che alla cartella indicata ci sia davvero un repository Git.
/// Restituisce il percorso della radice del repository (la cartella che
/// contiene `.git`), così il frontend salva sempre il percorso "giusto".
pub fn apri_repo(percorso: &str) -> Result<String, String> {
    let repo = Repository::discover(Path::new(percorso)).map_err(|e| e.to_string())?;
    radice(&repo)
}

/// Inizializza un nuovo repository (git init) nella cartella indicata.
pub fn init(percorso: &str) -> Result<String, String> {
    let repo = Repository::init(percorso).map_err(|e| e.to_string())?;
    radice(&repo)
}

/// Clona un repository remoto in una cartella locale.
pub fn clona(url: &str, destinazione: &str) -> Result<String, String> {
    // Le credenziali (per repo privati) sono gestite dai callback del modulo remote.
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(crate::remote::credenziali);

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    let repo = builder
        .clone(url, Path::new(destinazione))
        .map_err(|e| e.to_string())?;
    radice(&repo)
}

/// Percorso della radice (la cartella di lavoro) del repository.
fn radice(repo: &Repository) -> Result<String, String> {
    repo.workdir()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "repository senza cartella di lavoro (bare)".into())
}

/// Legge lo stato completo del repository: ramo corrente, file modificati
/// (divisi tra in stage e non), e quanto siamo avanti/indietro col remoto.
pub fn stato(percorso: &str) -> Result<StatoRepo, String> {
    let repo = crate::apri(percorso)?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true);

    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| e.to_string())?;

    let mut in_stage = Vec::new();
    let mut non_in_stage = Vec::new();

    for voce in statuses.iter() {
        let s = voce.status();
        let percorso_file = voce.path().unwrap_or("?").to_string();

        // Una stessa modifica può comparire sia in stage sia fuori
        // (es. modifico, aggiungo, poi modifico ancora): le trattiamo separate.
        if let Some(stato) = stato_in_stage(s) {
            in_stage.push(FileModificato {
                percorso: percorso_file.clone(),
                stato,
                in_stage: true,
            });
        }
        if let Some(stato) = stato_fuori_stage(s) {
            non_in_stage.push(FileModificato {
                percorso: percorso_file,
                stato,
                in_stage: false,
            });
        }
    }

    let vuoto = repo.head().is_err();
    let ramo = nome_ramo(&repo);
    let (avanti, indietro) = avanti_indietro(&repo).unwrap_or((0, 0));

    Ok(StatoRepo {
        ramo,
        in_stage,
        non_in_stage,
        avanti,
        indietro,
        vuoto,
    })
}

/// Traduce i flag "in staging" (INDEX_*) nel nostro StatoFile.
fn stato_in_stage(s: Status) -> Option<StatoFile> {
    if s.contains(Status::CONFLICTED) {
        return Some(StatoFile::Conflitto);
    }
    if s.contains(Status::INDEX_NEW) {
        Some(StatoFile::Nuovo)
    } else if s.contains(Status::INDEX_MODIFIED) {
        Some(StatoFile::Modificato)
    } else if s.contains(Status::INDEX_DELETED) {
        Some(StatoFile::Cancellato)
    } else if s.contains(Status::INDEX_RENAMED) {
        Some(StatoFile::Rinominato)
    } else if s.contains(Status::INDEX_TYPECHANGE) {
        Some(StatoFile::TipoCambiato)
    } else {
        None
    }
}

/// Traduce i flag "nella cartella di lavoro" (WT_*) nel nostro StatoFile.
fn stato_fuori_stage(s: Status) -> Option<StatoFile> {
    if s.contains(Status::CONFLICTED) {
        return Some(StatoFile::Conflitto);
    }
    if s.contains(Status::WT_NEW) {
        Some(StatoFile::Nuovo)
    } else if s.contains(Status::WT_MODIFIED) {
        Some(StatoFile::Modificato)
    } else if s.contains(Status::WT_DELETED) {
        Some(StatoFile::Cancellato)
    } else if s.contains(Status::WT_RENAMED) {
        Some(StatoFile::Rinominato)
    } else if s.contains(Status::WT_TYPECHANGE) {
        Some(StatoFile::TipoCambiato)
    } else {
        None
    }
}

/// Nome del ramo corrente; "(testa staccata)" se siamo su un commit diretto.
pub fn nome_ramo(repo: &Repository) -> String {
    match repo.head() {
        Ok(head) => {
            if head.is_branch() {
                head.shorthand().unwrap_or("?").to_string()
            } else {
                "(testa staccata)".into()
            }
        }
        // Nessun commit ancora: il ramo predefinito che verrà creato.
        Err(_) => "main".into(),
    }
}

/// Quanti commit il ramo locale è avanti/indietro rispetto al suo upstream.
fn avanti_indietro(repo: &Repository) -> Option<(usize, usize)> {
    let head = repo.head().ok()?;
    let nome = head.shorthand()?;
    let locale = repo.find_branch(nome, BranchType::Local).ok()?;
    let upstream = locale.upstream().ok()?;

    let oid_locale = locale.get().target()?;
    let oid_remoto = upstream.get().target()?;

    repo.graph_ahead_behind(oid_locale, oid_remoto).ok()
}
