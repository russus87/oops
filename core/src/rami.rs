//! Rami: elenco, creazione, checkout, eliminazione e merge.

use git2::build::CheckoutBuilder;
use git2::{BranchType, Repository};

use crate::model::Ramo;

/// Elenca tutti i rami, prima i locali poi i remoti. Segna quello corrente.
pub fn lista(percorso: &str) -> Result<Vec<Ramo>, String> {
    let repo = crate::apri(percorso)?;
    let corrente = crate::repo::nome_ramo(&repo);

    let mut rami = Vec::new();
    aggiungi_rami(&repo, BranchType::Local, false, &corrente, &mut rami)?;
    aggiungi_rami(&repo, BranchType::Remote, true, &corrente, &mut rami)?;
    Ok(rami)
}

/// Aggiunge alla lista i rami di un certo tipo (locale o remoto).
fn aggiungi_rami(
    repo: &Repository,
    tipo: BranchType,
    remoto: bool,
    corrente: &str,
    fuori: &mut Vec<Ramo>,
) -> Result<(), String> {
    let rami = repo.branches(Some(tipo)).map_err(|e| e.to_string())?;
    for r in rami {
        let (ramo, _) = r.map_err(|e| e.to_string())?;
        if let Some(nome) = ramo.name().map_err(|e| e.to_string())? {
            fuori.push(Ramo {
                nome: nome.to_string(),
                corrente: !remoto && nome == corrente,
                remoto,
            });
        }
    }
    Ok(())
}

/// Crea un nuovo ramo a partire dal commit corrente (HEAD).
/// Se `cambia` è true, ci si sposta subito sul nuovo ramo.
pub fn crea(percorso: &str, nome: &str, cambia: bool) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let commit = repo
        .head()
        .map_err(|e| e.to_string())?
        .peel_to_commit()
        .map_err(|e| e.to_string())?;
    repo.branch(nome, &commit, false).map_err(|e| e.to_string())?;
    if cambia {
        checkout(percorso, nome)?;
    }
    Ok(())
}

/// Si sposta su un altro ramo (git checkout / git switch).
pub fn checkout(percorso: &str, nome: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;

    let (oggetto, riferimento) = repo
        .revparse_ext(nome)
        .map_err(|e| e.to_string())?;

    // Aggiorna la cartella di lavoro al contenuto del ramo.
    let mut co = CheckoutBuilder::new();
    co.safe();
    repo.checkout_tree(&oggetto, Some(&mut co))
        .map_err(|e| e.to_string())?;

    // Sposta HEAD sul ramo (o sul commit, se è un nome senza riferimento).
    match riferimento {
        Some(rif) => repo
            .set_head(rif.name().ok_or("riferimento senza nome")?)
            .map_err(|e| e.to_string()),
        None => repo.set_head_detached(oggetto.id()).map_err(|e| e.to_string()),
    }
}

/// Elimina un ramo locale.
pub fn elimina(percorso: &str, nome: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut ramo = repo
        .find_branch(nome, BranchType::Local)
        .map_err(|e| e.to_string())?;
    ramo.delete().map_err(|e| e.to_string())
}

/// Unisce un altro ramo in quello corrente (git merge).
/// Gestisce: già aggiornato, fast-forward e merge normale. In caso di conflitti
/// restituisce un errore chiaro (i file in conflitto restano da risolvere).
pub fn merge(percorso: &str, nome: &str) -> Result<String, String> {
    let repo = crate::apri(percorso)?;

    // Commit da unire (la punta dell'altro ramo).
    let riferimento = repo
        .find_branch(nome, BranchType::Local)
        .map_err(|e| e.to_string())?
        .into_reference();
    let annotato = repo
        .reference_to_annotated_commit(&riferimento)
        .map_err(|e| e.to_string())?;

    let (analisi, _) = repo.merge_analysis(&[&annotato]).map_err(|e| e.to_string())?;

    if analisi.is_up_to_date() {
        return Ok("già aggiornato".into());
    }

    if analisi.is_fast_forward() {
        // Sposta semplicemente il ramo corrente in avanti.
        let nome_head = repo
            .head()
            .map_err(|e| e.to_string())?
            .name()
            .ok_or("HEAD senza nome")?
            .to_string();
        let mut rif = repo.find_reference(&nome_head).map_err(|e| e.to_string())?;
        rif.set_target(annotato.id(), "fast-forward")
            .map_err(|e| e.to_string())?;
        repo.set_head(&nome_head).map_err(|e| e.to_string())?;
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .map_err(|e| e.to_string())?;
        return Ok("fast-forward".into());
    }

    // Merge vero e proprio: applica le modifiche nell'indice/cartella.
    repo.merge(&[&annotato], None, None).map_err(|e| e.to_string())?;

    let index = repo.index().map_err(|e| e.to_string())?;
    if index.has_conflicts() {
        return Err("merge con conflitti: risolvi i file e fai un commit".into());
    }

    // Niente conflitti: creiamo il commit di merge con due genitori.
    let firma = repo
        .signature()
        .or_else(|_| git2::Signature::now("Oops", "oops@local"))
        .map_err(|e| e.to_string())?;
    let albero_id = repo
        .index()
        .map_err(|e| e.to_string())?
        .write_tree()
        .map_err(|e| e.to_string())?;
    let albero = repo.find_tree(albero_id).map_err(|e| e.to_string())?;
    let nostro = repo.head().map_err(|e| e.to_string())?.peel_to_commit().map_err(|e| e.to_string())?;
    let loro = repo.find_commit(annotato.id()).map_err(|e| e.to_string())?;

    repo.commit(
        Some("HEAD"),
        &firma,
        &firma,
        &format!("Merge del ramo '{nome}'"),
        &albero,
        &[&nostro, &loro],
    )
    .map_err(|e| e.to_string())?;

    repo.cleanup_state().map_err(|e| e.to_string())?;
    Ok("merge completato".into())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn crea_e_elenca() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_str().unwrap();
        Repository::init(dir.path()).unwrap();
        fs::write(dir.path().join("a.txt"), "x").unwrap();
        crate::stage::aggiungi(p, "a.txt").unwrap();
        crate::commit::crea(p, "init", "T", "t@t.it").unwrap();

        crea(p, "sviluppo", false).unwrap();
        let rami = lista(p).unwrap();
        let nomi: Vec<&str> = rami.iter().map(|r| r.nome.as_str()).collect();
        assert!(nomi.contains(&"sviluppo"));
        // Uno solo dev'essere quello corrente.
        assert_eq!(rami.iter().filter(|r| r.corrente).count(), 1);
    }
}
