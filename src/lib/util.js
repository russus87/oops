// Piccole utility condivise dal frontend: avatar deterministici (senza rete),
// tempo relativo ("3 ore fa") e il calcolo del grafo dei commit a corsie.

// Colori delle corsie del grafo (riusati anche per gli avatar).
export const COLORI_CORSIE = [
  "#ff6b6b", "#5c7cfa", "#51cf66", "#ffd43b",
  "#cc5de8", "#22b8cf", "#ff922b", "#f06595",
];

// Hash semplice e stabile (FNV-1a) di una stringa: serve a scegliere un colore.
function hash(s) {
  let h = 0x811c9dc5;
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return h >>> 0;
}

// Iniziali da un nome (una o due lettere maiuscole).
export function iniziali(nome) {
  const parti = (nome || "?").trim().split(/\s+/).filter(Boolean);
  if (parti.length === 0) return "?";
  if (parti.length === 1) return parti[0].slice(0, 2).toUpperCase();
  return (parti[0][0] + parti[parti.length - 1][0]).toUpperCase();
}

// Colore di sfondo dell'avatar, deciso dall'email (o dal nome) in modo stabile.
export function coloreAvatar(chiave) {
  const palette = [
    "#ff6b6b", "#5c7cfa", "#51cf66", "#f59f00", "#cc5de8",
    "#22b8cf", "#ff922b", "#f06595", "#20c997", "#845ef7",
  ];
  return palette[hash((chiave || "").toLowerCase()) % palette.length];
}

// Estensione (minuscola) di un percorso file, "" se non c'è.
export function estensione(percorso) {
  const nome = (percorso || "").split(/[\\/]/).pop() || "";
  const i = nome.lastIndexOf(".");
  return i > 0 ? nome.slice(i + 1).toLowerCase() : "";
}

// Colore stabile per un'estensione (pallino "lingua" accanto al file).
export function coloreLingua(percorso) {
  const ext = estensione(percorso);
  return ext ? coloreAvatar("ext:" + ext) : "var(--testo2)";
}

// Tempo trascorso in forma breve e amichevole ("adesso", "3 h fa", "2 gg fa").
export function tempoRelativo(timestampSec) {
  if (!timestampSec) return "";
  const diff = Math.floor(Date.now() / 1000) - timestampSec;
  if (diff < 0) return "nel futuro";
  if (diff < 45) return "adesso";
  if (diff < 90) return "1 min fa";
  const min = Math.round(diff / 60);
  if (min < 60) return `${min} min fa`;
  const ore = Math.round(diff / 3600);
  if (ore < 24) return `${ore} h fa`;
  const gg = Math.round(diff / 86400);
  if (gg < 7) return `${gg} gg fa`;
  const sett = Math.round(diff / 604800);
  if (sett < 5) return `${sett} sett fa`;
  const mesi = Math.round(diff / 2629800);
  if (mesi < 12) return `${mesi} mesi fa`;
  return `${Math.round(diff / 31557600)} anni fa`;
}

// Estensioni immagine riconosciute per l'anteprima.
export const ESTENSIONI_IMG = ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico", "avif"];

// Tipo MIME da estensione (per le data-URI delle immagini).
export function mimeDa(percorso) {
  const e = estensione(percorso);
  const m = { jpg: "jpeg", svg: "svg+xml", ico: "x-icon" };
  return "image/" + (m[e] || e);
}

// Mini-renderer Markdown → HTML (nessuna libreria: gestisce titoli, grassetto,
// corsivo, codice, liste, citazioni, link e paragrafi). L'input viene prima
// "sfuggito" per non iniettare HTML arbitrario.
export function markdownToHtml(md) {
  const esc = (s) =>
    s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  const righe = (md || "").replace(/\r/g, "").split("\n");
  let out = [];
  let inCodice = false;
  let inLista = false;
  let buffer = [];
  const chiudiLista = () => { if (inLista) { out.push("</ul>"); inLista = false; } };
  const inline = (t) =>
    esc(t)
      .replace(/`([^`]+)`/g, "<code>$1</code>")
      .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
      .replace(/(^|[^*])\*([^*]+)\*/g, "$1<em>$2</em>")
      .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" rel="noreferrer">$1</a>');

  for (const r of righe) {
    if (r.trim().startsWith("```")) {
      if (inCodice) { out.push("<pre class='md-code'>" + esc(buffer.join("\n")) + "</pre>"); buffer = []; inCodice = false; }
      else { chiudiLista(); inCodice = true; }
      continue;
    }
    if (inCodice) { buffer.push(r); continue; }
    const h = r.match(/^(#{1,6})\s+(.*)$/);
    if (h) { chiudiLista(); out.push(`<h${h[1].length}>${inline(h[2])}</h${h[1].length}>`); continue; }
    if (/^\s*[-*+]\s+/.test(r)) {
      if (!inLista) { out.push("<ul>"); inLista = true; }
      out.push("<li>" + inline(r.replace(/^\s*[-*+]\s+/, "")) + "</li>");
      continue;
    }
    if (/^\s*>\s?/.test(r)) { chiudiLista(); out.push("<blockquote>" + inline(r.replace(/^\s*>\s?/, "")) + "</blockquote>"); continue; }
    if (r.trim() === "") { chiudiLista(); continue; }
    chiudiLista();
    out.push("<p>" + inline(r) + "</p>");
  }
  chiudiLista();
  if (inCodice) out.push("<pre class='md-code'>" + esc(buffer.join("\n")) + "</pre>");
  return out.join("\n");
}

// --- Grafo dei commit a corsie ---
//
// Data la lista dei commit (dal più recente al più vecchio), assegna a ciascuno
// una "corsia" (colonna) seguendo i genitori e produce le istruzioni per
// disegnare, riga per riga, un grafo continuo con curve tra i nodi.
//
// Ogni riga restituita ha:
//   col        colonna del nodo (0-based)
//   colore     colore del nodo/della sua corsia
//   larghezza  numero di corsie attive nella riga (per dimensionare l'SVG)
//   segmenti   linee da disegnare: { x1,y1,x2,y2, colore } in coordinate
//              normalizzate (x = indice colonna, y = 0 in alto .. 1 in basso;
//              il nodo è a y = 0.5). Il chiamante scala su pixel.
export function calcolaGrafo(commits) {
  const lanes = []; // id breve del prossimo commit atteso in ogni corsia (o null)
  const out = [];

  for (const c of commits) {
    const sid = c.id_breve;
    const primaAttive = lanes.map((l) => l !== null);

    // Colonne che aspettavano questo commit: qui convergono (merge di corsie).
    const incoming = [];
    lanes.forEach((l, i) => l === sid && incoming.push(i));

    // Colonna del nodo: la prima che lo attendeva, altrimenti una corsia libera.
    let col;
    if (incoming.length) {
      col = incoming[0];
    } else {
      col = lanes.indexOf(null);
      if (col < 0) {
        col = lanes.length;
        lanes.push(null);
      }
    }
    const colore = COLORI_CORSIE[col % COLORI_CORSIE.length];

    // Libera le altre corsie che convergevano su questo nodo.
    for (let k = 1; k < incoming.length; k++) lanes[incoming[k]] = null;

    // Genitori: il primo continua nella corsia del nodo, gli altri aprono corsie.
    const genitori = c.genitori || [];
    const colonneGenitori = [];
    if (genitori.length) {
      lanes[col] = genitori[0];
      colonneGenitori.push(col);
      for (let k = 1; k < genitori.length; k++) {
        let libera = lanes.indexOf(null);
        if (libera < 0) {
          libera = lanes.length;
          lanes.push(null);
        }
        lanes[libera] = genitori[k];
        colonneGenitori.push(libera);
      }
    } else {
      lanes[col] = null; // commit radice: la corsia si chiude
    }

    // Costruisce i segmenti da disegnare per questa riga.
    const segmenti = [];
    const colColore = (i) => COLORI_CORSIE[i % COLORI_CORSIE.length];

    // 1) Corsie che passano dritte attraverso la riga (non toccano il nodo).
    primaAttive.forEach((attiva, i) => {
      if (!attiva) return;
      if (i === col) return;
      if (incoming.includes(i)) return; // gestita come merge entrante
      segmenti.push({ x1: i, y1: 0, x2: i, y2: 1, colore: colColore(i) });
    });

    // 2) Metà superiore della corsia del nodo (se veniva da sopra).
    if (primaAttive[col] && !incoming.includes(col)) {
      // caso raro: la corsia esisteva ma con altro id; la trattiamo come passante
      segmenti.push({ x1: col, y1: 0, x2: col, y2: 1, colore });
    }
    if (incoming.includes(col)) {
      segmenti.push({ x1: col, y1: 0, x2: col, y2: 0.5, colore });
    }

    // 3) Merge entranti da altre colonne: curva dall'alto verso il nodo.
    incoming.forEach((i) => {
      if (i === col) return;
      segmenti.push({ x1: i, y1: 0, x2: col, y2: 0.5, colore: colColore(i) });
    });

    // 4) Uscite verso i genitori: curva dal nodo verso la colonna del genitore.
    colonneGenitori.forEach((cp) => {
      segmenti.push({ x1: col, y1: 0.5, x2: cp, y2: 1, colore: colColore(cp) });
    });

    const larghezza = lanes.reduce((m, l, i) => (l !== null ? i + 1 : m), col + 1);
    out.push({ col, colore, larghezza, segmenti, merge: (c.genitori || []).length > 1 });
  }
  return out;
}
