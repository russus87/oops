//! Blame: per ogni riga di un file, chi l'ha toccata l'ultima volta.

use std::path::Path;

use crate::model::VoceBlame;

/// Calcola il blame di un file (versione nella cartella di lavoro).
pub fn blame(percorso: &str, file: &str) -> Result<Vec<VoceBlame>, String> {
    let repo = crate::apri(percorso)?;
    let blame = repo
        .blame_file(Path::new(file), None)
        .map_err(|e| e.to_string())?;

    // Leggiamo il contenuto del file per accostare il testo a ogni riga.
    let assoluto = repo
        .workdir()
        .ok_or("repository senza cartella di lavoro")?
        .join(file);
    let contenuto = std::fs::read_to_string(&assoluto).map_err(|e| e.to_string())?;

    let mut righe = Vec::new();
    for (i, testo) in contenuto.lines().enumerate() {
        let numero = i + 1;
        let (id_breve, autore) = match blame.get_line(numero) {
            Some(hunk) => {
                let id = hunk.final_commit_id().to_string();
                let firma = hunk.final_signature();
                (
                    id.chars().take(7).collect::<String>(),
                    firma.name().unwrap_or("?").to_string(),
                )
            }
            None => (String::new(), String::new()),
        };
        righe.push(VoceBlame {
            numero,
            id_breve,
            autore,
            testo: testo.to_string(),
        });
    }
    Ok(righe)
}
