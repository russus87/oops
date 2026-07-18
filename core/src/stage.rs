//! Area di staging: mettere/togliere file e scartare le modifiche.
//! Sono le operazioni dietro ai pulsanti "+" / "−" / "scarta" della UI.

use std::path::Path;

use git2::build::CheckoutBuilder;
use git2::IndexAddOption;

/// Mette un file in staging (git add). Gestisce anche i file cancellati:
/// `add_all` toglie dall'indice i file spariti dalla cartella.
pub fn aggiungi(percorso: &str, file: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut index = repo.index().map_err(|e| e.to_string())?;
    index
        .add_all([file].iter(), IndexAddOption::DEFAULT, None)
        .map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())
}

/// Mette in staging TUTTI i file modificati (git add -A).
pub fn aggiungi_tutto(percorso: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut index = repo.index().map_err(|e| e.to_string())?;
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())
}

/// Toglie un file dallo staging lasciando le modifiche nella cartella
/// (git restore --staged). Riporta la voce dell'indice a com'è in HEAD.
pub fn togli(percorso: &str, file: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let head = repo.head().ok();

    match head {
        // C'è almeno un commit: riportiamo il file allo stato di HEAD.
        Some(head) => {
            let oggetto = head.peel(git2::ObjectType::Commit).map_err(|e| e.to_string())?;
            repo.reset_default(Some(&oggetto), [file])
                .map_err(|e| e.to_string())
        }
        // Repo senza commit: togliere dallo stage = togliere dall'indice.
        None => {
            let mut index = repo.index().map_err(|e| e.to_string())?;
            index.remove_path(Path::new(file)).map_err(|e| e.to_string())?;
            index.write().map_err(|e| e.to_string())
        }
    }
}

/// Toglie dallo stage tutti i file (git reset).
pub fn togli_tutto(percorso: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let head = repo.head().ok();
    match head {
        Some(head) => {
            let oggetto = head.peel(git2::ObjectType::Commit).map_err(|e| e.to_string())?;
            // reset "mixed": HEAD invariato, indice riportato a HEAD.
            repo.reset(&oggetto, git2::ResetType::Mixed, None)
                .map_err(|e| e.to_string())
        }
        None => {
            let mut index = repo.index().map_err(|e| e.to_string())?;
            index.clear().map_err(|e| e.to_string())?;
            index.write().map_err(|e| e.to_string())
        }
    }
}

/// Scarta le modifiche di un file nella cartella di lavoro (git restore <file>).
/// Riporta il file a com'è nell'indice. ATTENZIONE: operazione distruttiva.
pub fn scarta(percorso: &str, file: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut co = CheckoutBuilder::new();
    co.force().path(file).update_index(false);
    repo.checkout_index(None, Some(&mut co))
        .map_err(|e| e.to_string())
}

/// Scarta TUTTE le modifiche dei file tracciati (git checkout -- .).
/// Distruttivo: i file non tracciati restano (vedi `pulisci_non_tracciati`).
pub fn scarta_tutto(percorso: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut co = CheckoutBuilder::new();
    co.force().remove_untracked(false);
    repo.checkout_index(None, Some(&mut co))
        .map_err(|e| e.to_string())
}

/// Cancella dal disco i file non ancora tracciati (git clean -f). Distruttivo.
pub fn pulisci_non_tracciati(percorso: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);
    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| e.to_string())?;

    let radice = repo
        .workdir()
        .ok_or("repository senza cartella di lavoro")?
        .to_path_buf();

    for voce in statuses.iter() {
        if voce.status().contains(git2::Status::WT_NEW) {
            if let Some(p) = voce.path() {
                let _ = std::fs::remove_file(radice.join(p));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    /// Crea un repo temporaneo, aggiunge un file e verifica che finisca in stage.
    #[test]
    fn aggiungi_e_togli() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_str().unwrap();
        git2::Repository::init(dir.path()).unwrap();

        fs::write(dir.path().join("ciao.txt"), "ciao").unwrap();
        aggiungi(p, "ciao.txt").unwrap();

        let stato = crate::repo::stato(p).unwrap();
        assert_eq!(stato.in_stage.len(), 1);
        assert_eq!(stato.in_stage[0].percorso, "ciao.txt");

        togli(p, "ciao.txt").unwrap();
        let stato = crate::repo::stato(p).unwrap();
        assert!(stato.in_stage.is_empty());
        assert_eq!(stato.non_in_stage.len(), 1);
    }
}
