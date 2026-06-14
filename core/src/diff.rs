//! Differenze (diff) di un file o di un commit, restituite come testo unificato
//! (lo stesso formato di `git diff`), che il frontend colora.

use git2::{ApplyLocation, ApplyOptions, Delta, Diff, DiffFormat, DiffOptions};

use crate::model::{FileModificato, StatoFile};

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

/// Elenca i file toccati da un commit (rispetto al primo genitore).
pub fn lista_file_commit(percorso: &str, id: &str) -> Result<Vec<FileModificato>, String> {
    let repo = crate::apri(percorso)?;
    let oid = git2::Oid::from_str(id).map_err(|e| e.to_string())?;
    let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
    let albero = commit.tree().map_err(|e| e.to_string())?;
    let albero_padre = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let diff = repo
        .diff_tree_to_tree(albero_padre.as_ref(), Some(&albero), None)
        .map_err(|e| e.to_string())?;

    let mut file = Vec::new();
    for delta in diff.deltas() {
        let percorso_file = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "?".into());
        file.push(FileModificato {
            percorso: percorso_file,
            stato: da_delta(delta.status()),
            in_stage: false,
        });
    }
    Ok(file)
}

/// Diff di un singolo file dentro un commit (rispetto al primo genitore).
pub fn commit_file(percorso: &str, id: &str, file: &str) -> Result<String, String> {
    let repo = crate::apri(percorso)?;
    let oid = git2::Oid::from_str(id).map_err(|e| e.to_string())?;
    let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
    let albero = commit.tree().map_err(|e| e.to_string())?;
    let albero_padre = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let mut opts = DiffOptions::new();
    opts.pathspec(file);
    let diff = repo
        .diff_tree_to_tree(albero_padre.as_ref(), Some(&albero), Some(&mut opts))
        .map_err(|e| e.to_string())?;
    in_testo(&diff)
}

/// Mette in stage (o toglie dallo stage, se `in_stage`) UN SOLO hunk di un file.
/// `indice` è la posizione del hunk nel diff del file (0 = il primo).
pub fn hunk_stage(percorso: &str, file: &str, indice: usize, in_stage: bool) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut opts = DiffOptions::new();
    opts.pathspec(file);

    let diff = if in_stage {
        // Togliere dallo stage: diff HEAD→indice rovesciato, applicato all'indice.
        let albero = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
        opts.reverse(true);
        repo.diff_tree_to_index(albero.as_ref(), None, Some(&mut opts))
            .map_err(|e| e.to_string())?
    } else {
        // Mettere in stage: diff indice→cartella, applicato all'indice.
        opts.include_untracked(true).recurse_untracked_dirs(true);
        repo.diff_index_to_workdir(None, Some(&mut opts))
            .map_err(|e| e.to_string())?
    };
    applica_hunk(&repo, &diff, indice, ApplyLocation::Index)
}

/// Scarta UN SOLO hunk delle modifiche non in stage di un file (distruttivo).
pub fn hunk_scarta(percorso: &str, file: &str, indice: usize) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut opts = DiffOptions::new();
    opts.pathspec(file).reverse(true);
    // Diff indice→cartella rovesciato, applicato alla cartella: annulla il hunk.
    let diff = repo
        .diff_index_to_workdir(None, Some(&mut opts))
        .map_err(|e| e.to_string())?;
    applica_hunk(&repo, &diff, indice, ApplyLocation::WorkDir)
}

/// Applica al repository solo l'n-esimo hunk del diff.
fn applica_hunk(
    repo: &git2::Repository,
    diff: &Diff,
    indice: usize,
    dove: ApplyLocation,
) -> Result<(), String> {
    let mut contatore = 0usize;
    let mut opts = ApplyOptions::new();
    opts.hunk_callback(|_hunk| {
        let prendi = contatore == indice;
        contatore += 1;
        prendi
    });
    repo.apply(diff, dove, Some(&mut opts)).map_err(|e| e.to_string())
}

/// Traduce lo stato di un delta (file in un diff) nel nostro StatoFile.
fn da_delta(d: Delta) -> StatoFile {
    match d {
        Delta::Added | Delta::Untracked => StatoFile::Nuovo,
        Delta::Deleted => StatoFile::Cancellato,
        Delta::Renamed | Delta::Copied => StatoFile::Rinominato,
        Delta::Typechange => StatoFile::TipoCambiato,
        Delta::Conflicted => StatoFile::Conflitto,
        _ => StatoFile::Modificato,
    }
}

/// Trasforma un Diff di git2 in testo unificato (con +/-/@@ come git).
pub(crate) fn in_testo(diff: &Diff) -> Result<String, String> {
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
