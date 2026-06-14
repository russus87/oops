//! Differenze (diff) di un file o di un commit, restituite come testo unificato
//! (lo stesso formato di `git diff`), che il frontend colora.

use git2::{Diff, DiffFormat, DiffOptions};

/// Diff di un singolo file. Se `in_stage` è true mostra le modifiche già in
/// staging (indice vs HEAD); altrimenti quelle nella cartella (cartella vs indice).
pub fn file(percorso: &str, file: &str, in_stage: bool) -> Result<String, String> {
    let repo = crate::apri(percorso)?;

    let mut opts = DiffOptions::new();
    opts.pathspec(file);
    // Mostra anche i file nuovi per intero (altrimenti non si vedrebbero).
    opts.include_untracked(true).recurse_untracked_dirs(true);

    let diff = if in_stage {
        let albero = repo
            .head()
            .ok()
            .and_then(|h| h.peel_to_tree().ok());
        repo.diff_tree_to_index(albero.as_ref(), None, Some(&mut opts))
            .map_err(|e| e.to_string())?
    } else {
        repo.diff_index_to_workdir(None, Some(&mut opts))
            .map_err(|e| e.to_string())?
    };

    in_testo(&diff)
}

/// Diff di un intero commit rispetto al suo primo genitore (cosa ha cambiato).
pub fn commit(percorso: &str, id: &str) -> Result<String, String> {
    let repo = crate::apri(percorso)?;
    let oid = git2::Oid::from_str(id).map_err(|e| e.to_string())?;
    let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
    let albero = commit.tree().map_err(|e| e.to_string())?;

    // Albero del genitore (per il primo commit non c'è: confronto col vuoto).
    let albero_padre = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let diff = repo
        .diff_tree_to_tree(albero_padre.as_ref(), Some(&albero), None)
        .map_err(|e| e.to_string())?;
    in_testo(&diff)
}

/// Trasforma un Diff di git2 in testo unificato (con +/-/@@ come git).
fn in_testo(diff: &Diff) -> Result<String, String> {
    let mut out = String::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        // I caratteri di contesto/aggiunta/rimozione vanno premessi alla riga.
        match line.origin() {
            '+' | '-' | ' ' => out.push(line.origin()),
            _ => {}
        }
        out.push_str(&String::from_utf8_lossy(line.content()));
        true
    })
    .map_err(|e| e.to_string())?;
    Ok(out)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn diff_file_modificato() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_str().unwrap();
        git2::Repository::init(dir.path()).unwrap();
        fs::write(dir.path().join("a.txt"), "riga uno\n").unwrap();
        crate::stage::aggiungi(p, "a.txt").unwrap();
        crate::commit::crea(p, "init", "T", "t@t.it").unwrap();

        // Modifico il file e controllo che il diff contenga la nuova riga.
        fs::write(dir.path().join("a.txt"), "riga uno\nriga due\n").unwrap();
        let d = file(p, "a.txt", false).unwrap();
        assert!(d.contains("+riga due"));
    }
}
