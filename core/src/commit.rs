//! Cronologia (log) e creazione di commit.

use std::collections::HashMap;

use git2::{Repository, Signature, Sort};

use crate::model::VoceLog;

/// Legge la cronologia dei commit a partire da HEAD (i più recenti prima).
/// `limite` = quanti commit al massimo restituire.
pub fn log(percorso: &str, limite: usize) -> Result<Vec<VoceLog>, String> {
    let repo = crate::apri(percorso)?;

    // Repo senza commit: cronologia vuota, non è un errore.
    if repo.head().is_err() {
        return Ok(Vec::new());
    }

    let decorazioni = mappa_riferimenti(&repo);

    let mut walk = repo.revwalk().map_err(|e| e.to_string())?;
    walk.push_head().map_err(|e| e.to_string())?;
    walk.set_sorting(Sort::TIME).map_err(|e| e.to_string())?;

    let mut voci = Vec::new();
    for oid in walk.take(limite) {
        let oid = oid.map_err(|e| e.to_string())?;
        let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
        voci.push(in_voce(&commit, &decorazioni));
    }
    Ok(voci)
}

/// Cronologia dei soli commit che hanno toccato un certo file.
pub fn log_file(percorso: &str, file: &str, limite: usize) -> Result<Vec<VoceLog>, String> {
    let repo = crate::apri(percorso)?;
    if repo.head().is_err() {
        return Ok(Vec::new());
    }
    let decorazioni = mappa_riferimenti(&repo);

    let mut walk = repo.revwalk().map_err(|e| e.to_string())?;
    walk.push_head().map_err(|e| e.to_string())?;
    walk.set_sorting(Sort::TIME).map_err(|e| e.to_string())?;

    let percorso_file = std::path::Path::new(file);
    let mut voci = Vec::new();
    for oid in walk {
        if voci.len() >= limite {
            break;
        }
        let oid = oid.map_err(|e| e.to_string())?;
        let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
        if tocca_file(&repo, &commit, percorso_file) {
            voci.push(in_voce(&commit, &decorazioni));
        }
    }
    Ok(voci)
}

/// Vero se il commit ha modificato `file` rispetto al primo genitore.
fn tocca_file(repo: &Repository, commit: &git2::Commit, file: &std::path::Path) -> bool {
    let albero = match commit.tree() {
        Ok(t) => t,
        Err(_) => return false,
    };
    let albero_padre = commit.parent(0).ok().and_then(|p| p.tree().ok());
    let mut opts = git2::DiffOptions::new();
    opts.pathspec(file);
    match repo.diff_tree_to_tree(albero_padre.as_ref(), Some(&albero), Some(&mut opts)) {
        Ok(d) => d.deltas().len() > 0,
        Err(_) => false,
    }
}

/// Costruisce una mappa "id commit -> nomi dei rami/tag che lo puntano".
fn mappa_riferimenti(repo: &Repository) -> HashMap<String, Vec<String>> {
    let mut mappa: HashMap<String, Vec<String>> = HashMap::new();
    if let Ok(riferimenti) = repo.references() {
        for r in riferimenti.flatten() {
            // Risolviamo fino al commit (le tag annotate puntano a un oggetto tag).
            if let Ok(commit) = r.peel_to_commit() {
                let nome = r
                    .shorthand()
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                if nome.is_empty() || nome == "HEAD" {
                    continue;
                }
                mappa.entry(commit.id().to_string()).or_default().push(nome);
            }
        }
    }
    mappa
}

/// Trasforma un commit di git2 nella nostra VoceLog (con data leggibile).
fn in_voce(commit: &git2::Commit, decorazioni: &HashMap<String, Vec<String>>) -> VoceLog {
    let autore = commit.author();
    let id = commit.id().to_string();
    let riferimenti = decorazioni.get(&id).cloned().unwrap_or_default();
    VoceLog {
        id_breve: id.chars().take(7).collect(),
        titolo: commit.summary().unwrap_or("(senza messaggio)").to_string(),
        autore: autore.name().unwrap_or("?").to_string(),
        email: autore.email().unwrap_or("").to_string(),
        data: data_leggibile(commit.time().seconds()),
        genitori: commit
            .parent_ids()
            .map(|p| p.to_string().chars().take(7).collect())
            .collect(),
        riferimenti,
        id,
    }
}

/// Converte un timestamp Unix in "AAAA-MM-GG HH:MM" (ora locale approssimata UTC).
/// Calcolo manuale per non aggiungere dipendenze (chrono) al core.
fn data_leggibile(secondi: i64) -> String {
    // Giorni dall'epoca e secondi nel giorno.
    let giorni = secondi.div_euclid(86_400);
    let resto = secondi.rem_euclid(86_400);
    let (ore, min) = (resto / 3600, (resto % 3600) / 60);

    // Algoritmo civile (Howard Hinnant) per data da giorni dall'epoca.
    let z = giorni + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let anno = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let giorno = doy - (153 * mp + 2) / 5 + 1;
    let mese = if mp < 10 { mp + 3 } else { mp - 9 };
    let anno = if mese <= 2 { anno + 1 } else { anno };

    format!("{anno:04}-{mese:02}-{giorno:02} {ore:02}:{min:02}")
}

/// Crea un commit con i file attualmente in staging.
/// `nome`/`email` possono essere vuoti: in quel caso si usa la config di Git.
pub fn crea(percorso: &str, messaggio: &str, nome: &str, email: &str) -> Result<String, String> {
    if messaggio.trim().is_empty() {
        return Err("il messaggio del commit non può essere vuoto".into());
    }
    let repo = crate::apri(percorso)?;

    // Costruisce l'albero dai contenuti dell'indice (staging).
    let mut index = repo.index().map_err(|e| e.to_string())?;
    let albero_id = index.write_tree().map_err(|e| e.to_string())?;
    let albero = repo.find_tree(albero_id).map_err(|e| e.to_string())?;

    let firma = firma(&repo, nome, email)?;

    // Genitore = commit attuale (se esiste); altrimenti è il primo commit.
    let genitore = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok());
    let genitori: Vec<&git2::Commit> = genitore.iter().collect();

    let oid = repo
        .commit(Some("HEAD"), &firma, &firma, messaggio, &albero, &genitori)
        .map_err(|e| e.to_string())?;
    Ok(oid.to_string())
}

/// Modifica l'ultimo commit (git commit --amend): aggiorna l'albero con i file
/// attualmente in stage e, se `messaggio` non è vuoto, ne cambia il testo.
/// L'autore originale viene mantenuto.
pub fn amend(percorso: &str, messaggio: &str) -> Result<String, String> {
    let repo = crate::apri(percorso)?;
    let ultimo = repo
        .head()
        .map_err(|e| e.to_string())?
        .peel_to_commit()
        .map_err(|e| e.to_string())?;

    // Nuovo albero dai contenuti dello staging.
    let albero_id = repo
        .index()
        .map_err(|e| e.to_string())?
        .write_tree()
        .map_err(|e| e.to_string())?;
    let albero = repo.find_tree(albero_id).map_err(|e| e.to_string())?;

    let nuovo_msg = if messaggio.trim().is_empty() {
        None
    } else {
        Some(messaggio)
    };

    let oid = ultimo
        .amend(Some("HEAD"), None, None, None, nuovo_msg, Some(&albero))
        .map_err(|e| e.to_string())?;
    Ok(oid.to_string())
}

/// Condensa (squash) in un unico commit tutti i commit da `id_piu_vecchio`
/// fino a HEAD compresi, con un nuovo messaggio. Equivale a un soft reset al
/// genitore di quel commit seguito da un commit con l'intero contenuto.
/// Richiede che il commit più vecchio abbia un genitore (non il primo della storia).
pub fn condensa(percorso: &str, id_piu_vecchio: &str, messaggio: &str) -> Result<String, String> {
    if messaggio.trim().is_empty() {
        return Err("serve un messaggio per il commit condensato".into());
    }
    let repo = crate::apri(percorso)?;

    // Albero finale = quello dell'attuale HEAD (il risultato che vogliamo tenere).
    let testa = repo
        .head()
        .map_err(|e| e.to_string())?
        .peel_to_commit()
        .map_err(|e| e.to_string())?;
    let albero = testa.tree().map_err(|e| e.to_string())?;

    // Genitore del commit più vecchio: diventerà il genitore del commit condensato.
    let oid = git2::Oid::from_str(id_piu_vecchio).map_err(|e| e.to_string())?;
    let piu_vecchio = repo.find_commit(oid).map_err(|e| e.to_string())?;
    let padre_oid = piu_vecchio
        .parent_id(0)
        .map_err(|_| "non si può condensare includendo il primo commit".to_string())?;
    let padre = repo.find_commit(padre_oid).map_err(|e| e.to_string())?;

    // Sposta HEAD al genitore (soft: lascia stage e file come sono).
    repo.reset(padre.as_object(), git2::ResetType::Soft, None)
        .map_err(|e| e.to_string())?;

    let firma = repo
        .signature()
        .or_else(|_| Signature::now("Oops", "oops@local"))
        .map_err(|e| e.to_string())?;
    let nuovo = repo
        .commit(Some("HEAD"), &firma, &firma, messaggio, &albero, &[&padre])
        .map_err(|e| e.to_string())?;
    Ok(nuovo.to_string())
}

/// Messaggio dell'ultimo commit (per precompilare l'amend). Vuoto se non c'è.
pub fn ultimo_messaggio(percorso: &str) -> Result<String, String> {
    let repo = crate::apri(percorso)?;
    let commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    match commit {
        Some(c) => Ok(c.message().unwrap_or("").to_string()),
        None => Ok(String::new()),
    }
}

/// Prepara la firma (autore/committer). Se nome/email sono vuoti prova a
/// leggerli dalla configurazione di Git; come ultima spiaggia usa un default.
fn firma<'a>(repo: &Repository, nome: &'a str, email: &'a str) -> Result<Signature<'a>, String> {
    if !nome.trim().is_empty() && !email.trim().is_empty() {
        return Signature::now(nome, email).map_err(|e| e.to_string());
    }
    repo.signature()
        .or_else(|_| Signature::now("Oops", "oops@local"))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn primo_commit() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_str().unwrap();
        Repository::init(dir.path()).unwrap();

        fs::write(dir.path().join("a.txt"), "contenuto").unwrap();
        crate::stage::aggiungi(p, "a.txt").unwrap();
        crea(p, "primo commit", "Tester", "t@t.it").unwrap();

        let voci = log(p, 10).unwrap();
        assert_eq!(voci.len(), 1);
        assert_eq!(voci[0].titolo, "primo commit");
        assert_eq!(voci[0].autore, "Tester");
    }

    #[test]
    fn data_nota() {
        // 2021-01-01 00:00:00 UTC = 1609459200
        assert_eq!(data_leggibile(1_609_459_200), "2021-01-01 00:00");
    }
}
