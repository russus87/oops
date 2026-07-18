// Avvio dell'app Svelte: monta il componente principale dentro <div id="app">.
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";
import { stato } from "./lib/stato.svelte.js";

// Applica il tema salvato prima di mostrare l'interfaccia.
stato.applicaTema();

const app = mount(App, {
  target: document.getElementById("app"),
});

export default app;
