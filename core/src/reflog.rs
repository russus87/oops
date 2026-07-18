//! Reflog di HEAD: la cronologia dei movimenti del ramo (utile per "recuperare"
//! commit dopo reset/rebase sbagliati).

use crate::model::VoceReflog;

/// Legge il reflog di HEAD (dal più recente).
pub fn lista(percorso: &str) -> Result<Vec<VoceReflog>, String> {
    let repo = crate::apri(percorso)?;
    let reflog = repo.reflog("HEAD").map_err(|e| e.to_string())?;

    let mut voci = Vec::new();
    for i in 0..reflog.len() {
        if let Some(voce) = reflog.get(i) {
            voci.push(VoceReflog {
                id_breve: voce.id_new().to_string().chars().take(7).collect(),
                messaggio: voce.message().unwrap_or("").to_string(),
            });
        }
    }
    Ok(voci)
}
