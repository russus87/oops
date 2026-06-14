//! Elenco dei repository aperti di recente: un semplice file JSON.

use std::path::{Path, PathBuf};

use crate::model::RepoRecente;

/// Quanti repository recenti tenere al massimo.
const MAX: usize = 15;

/// Legge la lista dei repository recenti dal file (vuota se manca).
pub fn carica(file: &Path) -> Vec<RepoRecente> {
    std::fs::read_to_string(file)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

/// Salva la lista come JSON leggibile, creando la cartella se serve.
fn salva(file: &Path, lista: &[RepoRecente]) -> Result<(), String> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let testo = serde_json::to_string_pretty(lista).map_err(|e| e.to_string())?;
    std::fs::write(file, testo).map_err(|e| e.to_string())
}

/// Aggiunge (o sposta in cima) un repository alla lista dei recenti.
pub fn aggiungi(file: &Path, percorso: &str) -> Result<Vec<RepoRecente>, String> {
    let mut lista = carica(file);

    // Toglie eventuali duplicati dello stesso percorso.
    lista.retain(|r| r.percorso != percorso);

    let nome = PathBuf::from(percorso)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| percorso.to_string());

    lista.insert(
        0,
        RepoRecente {
            percorso: percorso.to_string(),
            nome,
        },
    );
    lista.truncate(MAX);
    salva(file, &lista)?;
    Ok(lista)
}

/// Toglie un repository dalla lista dei recenti.
pub fn rimuovi(file: &Path, percorso: &str) -> Result<Vec<RepoRecente>, String> {
    let mut lista = carica(file);
    lista.retain(|r| r.percorso != percorso);
    salva(file, &lista)?;
    Ok(lista)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn aggiungi_e_dedup() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("recenti.json");

        aggiungi(&file, "/a/progetto").unwrap();
        aggiungi(&file, "/b/altro").unwrap();
        // Riaggiunto: deve salire in cima senza duplicarsi.
        let lista = aggiungi(&file, "/a/progetto").unwrap();

        assert_eq!(lista.len(), 2);
        assert_eq!(lista[0].percorso, "/a/progetto");
        assert_eq!(lista[0].nome, "progetto");
    }
}
