//! Risoluzione dei conflitti dopo un merge/cherry-pick/rebase non riuscito.

use std::path::Path;

use git2::{ResetType, Status, StatusOptions};

use crate::model::ConflittoVersioni;

/// Restituisce le tre versioni di un file in conflitto (base/nostra/loro) più il
/// contenuto attuale della cartella (con i marcatori di conflitto), per l'editor
/// di merge a 3 vie.
pub fn versioni(percorso: &str, file: &str) -> Result<ConflittoVersioni, String> {
    let repo = crate::apri(percorso)?;
    let index = repo.index().map_err(|e| e.to_string())?;

    let mut base = String::new();
    let mut nostra = String::new();
    let mut loro = String::new();

    for c in index.conflicts().map_err(|e| e.to_string())? {
        let c = c.map_err(|e| e.to_string())?;
        let combacia = |v: &Option<git2::IndexEntry>| {
            v.as_ref()
                .map(|e| String::from_utf8_lossy(&e.path) == file)
                .unwrap_or(false)
        };
        if combacia(&c.our) || combacia(&c.their) || combacia(&c.ancestor) {
            base = contenuto_blob(&repo, &c.ancestor);
            nostra = contenuto_blob(&repo, &c.our);
            loro = contenuto_blob(&repo, &c.their);
            break;
        }
    }

    let corrente = repo
        .workdir()
        .map(|w| w.join(file))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();

    Ok(ConflittoVersioni { base, nostra, loro, corrente })
}

/// Legge il contenuto testuale del blob puntato da una voce dell'indice.
fn contenuto_blob(repo: &git2::Repository, voce: &Option<git2::IndexEntry>) -> String {
    voce
        .as_ref()
        .and_then(|e| repo.find_blob(e.id).ok())
        .map(|b| String::from_utf8_lossy(b.content()).to_string())
        .unwrap_or_default()
}

/// Salva il contenuto risolto di un file e lo segna come risolto (in stage).
pub fn salva(percorso: &str, file: &str, contenuto: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let assoluto = repo
        .workdir()
        .ok_or("repository senza cartella di lavoro")?
        .join(file);
    std::fs::write(&assoluto, contenuto).map_err(|e| e.to_string())?;
    let mut index = repo.index().map_err(|e| e.to_string())?;
    index.add_path(Path::new(file)).map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())
}

/// Elenca i file attualmente in conflitto.
pub fn lista(percorso: &str) -> Result<Vec<String>, String> {
    let repo = crate::apri(percorso)?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(false);
    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| e.to_string())?;

    let mut file = Vec::new();
    for voce in statuses.iter() {
        if voce.status().contains(Status::CONFLICTED) {
            file.push(voce.path().unwrap_or("?").to_string());
        }
    }
    Ok(file)
}

/// Risolve un file in conflitto scegliendo una versione: `lato` = "nostra"
/// (la nostra modifica) o "loro" (quella in arrivo). Scrive la versione scelta
/// nella cartella e la mette in stage (togliendo il conflitto).
pub fn risolvi(percorso: &str, file: &str, lato: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut index = repo.index().map_err(|e| e.to_string())?;

    // Cerca la voce di conflitto del file e prende l'id del blob giusto.
    let mut blob_id = None;
    for c in index.conflicts().map_err(|e| e.to_string())? {
        let c = c.map_err(|e| e.to_string())?;
        let voce = match lato {
            "loro" => c.their,
            _ => c.our,
        };
        if let Some(v) = voce {
            if String::from_utf8_lossy(&v.path) == file {
                blob_id = Some(v.id);
                break;
            }
        }
    }
    let blob_id = blob_id.ok_or_else(|| format!("nessun conflitto per {file}"))?;

    // Scrive il contenuto scelto nella cartella di lavoro.
    let blob = repo.find_blob(blob_id).map_err(|e| e.to_string())?;
    let assoluto = repo
        .workdir()
        .ok_or("repository senza cartella di lavoro")?
        .join(file);
    std::fs::write(&assoluto, blob.content()).map_err(|e| e.to_string())?;

    // Aggiungere il file all'indice risolve il conflitto.
    index.add_path(Path::new(file)).map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())
}

/// Segna come risolto un file usando il contenuto attuale nella cartella
/// (dopo che l'utente l'ha modificato a mano).
pub fn segna_risolto(percorso: &str, file: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut index = repo.index().map_err(|e| e.to_string())?;
    index.add_path(Path::new(file)).map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())
}

/// Annulla l'operazione in corso (merge/cherry-pick/rebase): riporta tutto
/// allo stato di HEAD e ripulisce lo stato del repository.
pub fn annulla(percorso: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let testa = repo
        .head()
        .map_err(|e| e.to_string())?
        .peel(git2::ObjectType::Commit)
        .map_err(|e| e.to_string())?;
    repo.reset(&testa, ResetType::Hard, None)
        .map_err(|e| e.to_string())?;
    repo.cleanup_state().map_err(|e| e.to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn conflitto_e_risoluzione() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_str().unwrap();
        git2::Repository::init(dir.path()).unwrap();

        // Base.
        fs::write(dir.path().join("f.txt"), "uno\n").unwrap();
        crate::stage::aggiungi(p, "f.txt").unwrap();
        crate::commit::crea(p, "base", "T", "t@t.it").unwrap();

        // Nome del ramo principale (master o main a seconda della config).
        let principale = {
            let r = git2::Repository::open(dir.path()).unwrap();
            let head = r.head().unwrap();
            let nome = head.shorthand().unwrap().to_string();
            nome
        };

        // Ramo "altro" cambia il file.
        crate::rami::crea(p, "altro", true).unwrap();
        fs::write(dir.path().join("f.txt"), "loro\n").unwrap();
        crate::stage::aggiungi(p, "f.txt").unwrap();
        crate::commit::crea(p, "loro", "T", "t@t.it").unwrap();

        // Torno sul principale e cambio lo stesso file in modo incompatibile.
        crate::rami::checkout(p, &principale).unwrap();
        fs::write(dir.path().join("f.txt"), "nostra\n").unwrap();
        crate::stage::aggiungi(p, "f.txt").unwrap();
        crate::commit::crea(p, "nostra", "T", "t@t.it").unwrap();

        // Merge -> conflitto.
        let esito = crate::rami::merge(p, "altro");
        assert!(esito.is_err());
        assert_eq!(lista(p).unwrap(), vec!["f.txt"]);

        // Risolvo prendendo "loro" e controllo che il conflitto sparisca.
        risolvi(p, "f.txt", "loro").unwrap();
        assert!(lista(p).unwrap().is_empty());
        assert_eq!(fs::read_to_string(dir.path().join("f.txt")).unwrap(), "loro\n");
    }
}
