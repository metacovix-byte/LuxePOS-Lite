# Changelog — LuxePOS Lite

Toutes les versions notables de LuxePOS Lite sont documentées ici.

Format inspiré de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/).
Versioning : [SemVer](https://semver.org/spec/v2.0.0.html).

## [1.0.0] — 2026-05-23 — MVP testeuses early access

### Ajouté
- Fork from LuxePOS v5.14.21 → version simplifiée pour 3 testeuses.
- Identifiant Tauri `ch.luxepos.lite` (n'écrase JAMAIS l'install LuxePOS principale).
- Repo séparé `metacovix-byte/LuxePOS-Lite` avec auto-updater Tauri signé
  (même clé minisign que LuxePOS).
- CI GitHub Actions adaptée : `release.yml` (Win + Mac + APK) sur tag `v*`,
  `deploy-pages.yml` (site demo).

### Inclus (MVP)
- Saisie produits + photos HD (~200 KB max 1200×1200)
- Import Excel intelligent (11 feuilles, parser hérité de LuxePOS v5.14.20)
- Multi-POS 3 emplacements (Atelier CHF, Annemasse EUR, Salon Genève CHF)
- Toggle Vendu/Pas vendu + date + clientName
- Clients (CRM léger) + Historique d'activité
- OneDrive sync chiffré AES-GCM 256 + backup local IDB
- Export Excel (.xlsx)
- Onboarding tutorial + Mode offline + queue sync
- Page Feedback utilisateurs (Formspree, 5-8 questions)
- Identité graphique LuxePOS conservée (glassmorphism, thème emerald)

### Hors-périmètre v1.0
POS / caisse / panier · WhatsApp · Instagram · SAV · Réparations · Lumi IA ·
BOM · Composants · Dashboard analytics complet.

### Tests
~10 tests Playwright (smoke + Excel + multi-POS), hérités et adaptés de
LuxePOS-Web/tests/smoke.spec.js.
