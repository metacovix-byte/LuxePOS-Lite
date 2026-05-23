# Changelog — LuxePOS Lite

Toutes les versions notables de LuxePOS Lite sont documentées ici.

Format inspiré de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/).
Versioning : [SemVer](https://semver.org/spec/v2.0.0.html).

## [1.0.1] — 2026-05-24 — Hotfix sécurité avant install testeuses

Audit indépendant a identifié 3 vulnérabilités critiques + 2 importantes
**avant** que les 3 testeuses installent. Tous corrigés avant tag v1.0.1.

### Sécurité — 3 critiques + 2 importantes
- 🔴 **CSP (Content Security Policy) ajoutée** dans `tauri.conf.json`.
  Avant : `csp: null` (porte grande ouverte). Après : liste blanche stricte
  des sources autorisées (self, gstatic pour Firebase, fonts Google).
- 🔴 **XSS — sites multiples corrigés** : `${product.name}`, `${client.email}`,
  `${item.name}` enveloppés dans `esc()` (helper qui échappe `<` `>` `"`).
  Empêche un nom client malicieux d'exécuter du JS dans la facture imprimée.
- 🔴 **Upload SVG retiré** du sélecteur de logo boutique. Les SVG peuvent
  contenir `<script>` — risque d'exécution si rendus dans certains contextes.
  Seuls PNG/JPG acceptés maintenant.
- 🟡 **Permissions Tauri réduites** (`capabilities/default.json`) — principle
  of least privilege. Retiré : `fs:default`, `process:default`, `shell:default`.
  Gardé : `dialog:default`, `shell:allow-open`, `updater:default`.
- 🟡 **Validation taille + extension Excel** dans `_readExcelFile()` :
  rejette fichier > 10 MB ou extension hors `.xlsx/.xls/.csv` (prévient DoS).

### Versions alignées
- `APP_CONFIG.VERSION` : `1.0.0` → `1.0.1`
- `package.json`, `Cargo.toml`, `tauri.conf.json` synchronisés à 1.0.1.

### Tests
5 smoke tests Playwright continuent de passer (boot OK, identité Lite,
multi-POS, CHF, SheetJS offline-first).

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
