//! Sottomoduli (submodule): elenco e aggiornamento.

use crate::model::Submodulo;

/// Elenca i sottomoduli del repository.
pub fn lista(percorso: &str) -> Result<Vec<Submodulo>, String> {
    let repo = crate::apri(percorso)?;
    let sub = repo.submodules().map_err(|e| e.to_string())?;
    Ok(sub
        .iter()
        .map(|s| Submodulo {
            nome: s.name().unwrap_or("?").to_string(),
            percorso: s.path().to_string_lossy().to_string(),
            url: s.url().unwrap_or("").to_string(),
        })
        .collect())
}

/// Aggiorna (clona/checkout) un sottomodulo, inizializzandolo se serve.
pub fn aggiorna(percorso: &str, nome: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut sub = repo.find_submodule(nome).map_err(|e| e.to_string())?;
    sub.update(true, None).map_err(|e| e.to_string())
}
