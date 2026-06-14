//! Tag (etichette): elenco, creazione (leggera o annotata) ed eliminazione.

use git2::{ObjectType, Signature};

use crate::model::Tag;

/// Elenca le tag del repository, con il commit puntato e l'eventuale messaggio.
pub fn lista(percorso: &str) -> Result<Vec<Tag>, String> {
    let repo = crate::apri(percorso)?;
    let nomi = repo.tag_names(None).map_err(|e| e.to_string())?;

    let mut tag = Vec::new();
    for nome in nomi.iter().flatten() {
        let oggetto = repo
            .revparse_single(&format!("refs/tags/{nome}"))
            .map_err(|e| e.to_string())?;

        // Se è un oggetto tag "annotato" ha messaggio e punta a un commit;
        // altrimenti è una tag leggera (un semplice riferimento a un commit).
        let (messaggio, id) = match oggetto.as_tag() {
            Some(t) => (
                t.message().unwrap_or("").trim().to_string(),
                t.target_id(),
            ),
            None => (String::new(), oggetto.id()),
        };

        tag.push(Tag {
            nome: nome.to_string(),
            messaggio,
            id_breve: id.to_string().chars().take(7).collect(),
        });
    }
    Ok(tag)
}

/// Crea una tag sul commit corrente (HEAD). Se `messaggio` non è vuoto crea una
/// tag annotata, altrimenti una leggera.
pub fn crea(percorso: &str, nome: &str, messaggio: &str) -> Result<(), String> {
    if nome.trim().is_empty() {
        return Err("il nome della tag non può essere vuoto".into());
    }
    let repo = crate::apri(percorso)?;
    let testa = repo
        .head()
        .map_err(|e| e.to_string())?
        .peel(ObjectType::Commit)
        .map_err(|e| e.to_string())?;

    if messaggio.trim().is_empty() {
        repo.tag_lightweight(nome, &testa, false)
            .map(|_| ())
            .map_err(|e| e.to_string())
    } else {
        let firma = repo
            .signature()
            .or_else(|_| Signature::now("Oops", "oops@local"))
            .map_err(|e| e.to_string())?;
        repo.tag(nome, &testa, &firma, messaggio, false)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

/// Elimina una tag.
pub fn elimina(percorso: &str, nome: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    repo.tag_delete(nome).map_err(|e| e.to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn crea_ed_elenca() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_str().unwrap();
        git2::Repository::init(dir.path()).unwrap();
        fs::write(dir.path().join("a.txt"), "x").unwrap();
        crate::stage::aggiungi(p, "a.txt").unwrap();
        crate::commit::crea(p, "init", "T", "t@t.it").unwrap();

        crea(p, "v1.0", "prima release").unwrap();
        crea(p, "leggera", "").unwrap();

        let tag = lista(p).unwrap();
        assert_eq!(tag.len(), 2);
        let v1 = tag.iter().find(|t| t.nome == "v1.0").unwrap();
        assert_eq!(v1.messaggio, "prima release");

        elimina(p, "leggera").unwrap();
        assert_eq!(lista(p).unwrap().len(), 1);
    }
}
