//! Lettura del contenuto dei file per le anteprime (markdown, immagini).
//! Il contenuto binario (immagini) viene restituito come base64, così il
//! frontend lo mostra con una data-URI senza dover accedere al filesystem.

/// Contenuto testuale di un file nella cartella di lavoro (per l'anteprima
/// markdown). Errore se il file non è testo UTF-8.
pub fn testo_lavoro(percorso: &str, file: &str) -> Result<String, String> {
    let repo = crate::apri(percorso)?;
    let radice = repo.workdir().ok_or("repository senza cartella di lavoro")?;
    let dati = std::fs::read(radice.join(file)).map_err(|e| e.to_string())?;
    String::from_utf8(dati).map_err(|_| "il file non è testo UTF-8".into())
}

/// Contenuto (base64) di un file nella cartella di lavoro, per l'anteprima
/// delle immagini. Vuoto se il file non esiste.
pub fn b64_lavoro(percorso: &str, file: &str) -> Result<String, String> {
    let repo = crate::apri(percorso)?;
    let radice = repo.workdir().ok_or("repository senza cartella di lavoro")?;
    match std::fs::read(radice.join(file)) {
        Ok(d) => Ok(base64(&d)),
        Err(_) => Ok(String::new()),
    }
}

/// Contenuto (base64) della versione del file in HEAD (per confrontare l'immagine
/// "prima/dopo"). Vuoto se il file non esiste in HEAD (file nuovo).
pub fn b64_head(percorso: &str, file: &str) -> Result<String, String> {
    let repo = crate::apri(percorso)?;
    let albero = match repo.head().ok().and_then(|h| h.peel_to_tree().ok()) {
        Some(t) => t,
        None => return Ok(String::new()),
    };
    let entry = match albero.get_path(std::path::Path::new(file)) {
        Ok(e) => e,
        Err(_) => return Ok(String::new()),
    };
    let dati = match repo.find_blob(entry.id()) {
        Ok(b) => b.content().to_vec(),
        Err(_) => return Ok(String::new()),
    };
    Ok(base64(&dati))
}

/// Codifica base64 standard (per le data-URI delle immagini).
fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        out.push(T[(b0 >> 2) as usize] as char);
        out.push(T[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            T[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[(b2 & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn base64_noto() {
        assert_eq!(base64(b"Man"), "TWFu");
        assert_eq!(base64(b"Ma"), "TWE=");
        assert_eq!(base64(b"M"), "TQ==");
    }
}
