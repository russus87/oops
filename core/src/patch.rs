//! Importazione/esportazione di patch (file .patch / .diff).

use git2::{ApplyLocation, Diff};

/// Esporta il diff di un commit in un file di testo (formato patch unificato).
pub fn esporta(percorso: &str, id: &str, file_destinazione: &str) -> Result<(), String> {
    let testo = crate::diff::commit(percorso, id, false)?;
    std::fs::write(file_destinazione, testo).map_err(|e| e.to_string())
}

/// Applica una patch (file .patch/.diff) alla cartella di lavoro.
pub fn applica(percorso: &str, file_patch: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let contenuto = std::fs::read(file_patch).map_err(|e| e.to_string())?;
    let diff = Diff::from_buffer(&contenuto).map_err(|e| e.to_string())?;
    repo.apply(&diff, ApplyLocation::WorkDir, None)
        .map_err(|e| e.to_string())
}
