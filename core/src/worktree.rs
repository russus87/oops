//! Worktree (alberi di lavoro aggiuntivi collegati allo stesso repository).

use std::path::Path;

use crate::model::VoceWorktree;

/// Elenca i worktree collegati.
pub fn lista(percorso: &str) -> Result<Vec<VoceWorktree>, String> {
    let repo = crate::apri(percorso)?;
    let nomi = repo.worktrees().map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for nome in nomi.iter().flatten() {
        let percorso_wt = repo
            .find_worktree(nome)
            .ok()
            .map(|w| w.path().to_string_lossy().to_string())
            .unwrap_or_default();
        out.push(VoceWorktree {
            nome: nome.to_string(),
            percorso: percorso_wt,
        });
    }
    Ok(out)
}

/// Crea un nuovo worktree con un nome e una cartella.
pub fn aggiungi(percorso: &str, nome: &str, cartella: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    repo.worktree(nome, Path::new(cartella), None)
        .map(|_| ())
        .map_err(|e| e.to_string())
}
