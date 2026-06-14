//! Stash: mettere da parte le modifiche correnti per riprenderle dopo.

use git2::{Signature, StashApplyOptions, StashFlags};

use crate::model::VoceStash;

/// Elenca gli stash presenti (il più recente ha indice 0).
pub fn lista(percorso: &str) -> Result<Vec<VoceStash>, String> {
    let mut repo = crate::apri(percorso)?;
    let mut voci = Vec::new();
    repo.stash_foreach(|indice, messaggio, _oid| {
        voci.push(VoceStash {
            indice,
            messaggio: messaggio.to_string(),
        });
        true
    })
    .map_err(|e| e.to_string())?;
    Ok(voci)
}

/// Mette da parte le modifiche correnti. Se `includi_non_tracciati` è true,
/// stasha anche i file nuovi (non ancora tracciati).
pub fn salva(percorso: &str, messaggio: &str, includi_non_tracciati: bool) -> Result<(), String> {
    let mut repo = crate::apri(percorso)?;
    let firma = repo
        .signature()
        .or_else(|_| Signature::now("Oops", "oops@local"))
        .map_err(|e| e.to_string())?;

    let mut flags = StashFlags::DEFAULT;
    if includi_non_tracciati {
        flags |= StashFlags::INCLUDE_UNTRACKED;
    }

    let msg = if messaggio.trim().is_empty() {
        None
    } else {
        Some(messaggio)
    };
    repo.stash_save2(&firma, msg, Some(flags))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Riapplica uno stash lasciandolo nella lista (git stash apply).
pub fn applica(percorso: &str, indice: usize) -> Result<(), String> {
    let mut repo = crate::apri(percorso)?;
    let mut opts = StashApplyOptions::new();
    repo.stash_apply(indice, Some(&mut opts))
        .map_err(|e| e.to_string())
}

/// Riapplica uno stash e lo toglie dalla lista (git stash pop).
pub fn pop(percorso: &str, indice: usize) -> Result<(), String> {
    let mut repo = crate::apri(percorso)?;
    let mut opts = StashApplyOptions::new();
    repo.stash_pop(indice, Some(&mut opts))
        .map_err(|e| e.to_string())
}

/// Elimina uno stash senza applicarlo (git stash drop).
pub fn elimina(percorso: &str, indice: usize) -> Result<(), String> {
    let mut repo = crate::apri(percorso)?;
    repo.stash_drop(indice).map_err(|e| e.to_string())
}

/// Diff di uno stash rispetto al suo punto di partenza (cosa contiene).
pub fn diff(percorso: &str, indice: usize) -> Result<String, String> {
    let mut repo = crate::apri(percorso)?;

    // Trova l'id del commit-stash corrispondente all'indice.
    let mut oid = None;
    repo.stash_foreach(|i, _msg, id| {
        if i == indice {
            oid = Some(*id);
            false
        } else {
            true
        }
    })
    .map_err(|e| e.to_string())?;
    let oid = oid.ok_or_else(|| format!("stash {indice} non trovato"))?;

    let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
    let albero = commit.tree().map_err(|e| e.to_string())?;
    // Il primo genitore dello stash è lo stato di partenza (la base).
    let base = commit
        .parent(0)
        .map_err(|e| e.to_string())?
        .tree()
        .map_err(|e| e.to_string())?;

    let diff = repo
        .diff_tree_to_tree(Some(&base), Some(&albero), None)
        .map_err(|e| e.to_string())?;
    crate::diff::in_testo(&diff)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn salva_e_applica() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_str().unwrap();
        git2::Repository::init(dir.path()).unwrap();
        fs::write(dir.path().join("a.txt"), "uno\n").unwrap();
        crate::stage::aggiungi(p, "a.txt").unwrap();
        crate::commit::crea(p, "init", "T", "t@t.it").unwrap();

        // Modifico e stasho.
        fs::write(dir.path().join("a.txt"), "uno\ndue\n").unwrap();
        salva(p, "lavoro a metà", false).unwrap();
        assert_eq!(lista(p).unwrap().len(), 1);
        // Dopo lo stash il file torna pulito.
        assert!(crate::repo::stato(p).unwrap().non_in_stage.is_empty());

        // Pop: la modifica torna e lo stash sparisce.
        pop(p, 0).unwrap();
        assert!(lista(p).unwrap().is_empty());
        assert_eq!(crate::repo::stato(p).unwrap().non_in_stage.len(), 1);
    }
}
