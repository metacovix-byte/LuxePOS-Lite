# Changelog — LuxePOS Lite

Toutes les versions notables de LuxePOS Lite sont documentées ici.

Format inspiré de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/).
Versioning : [SemVer](https://semver.org/spec/v2.0.0.html).

## [1.0.5] — 2026-05-26 — Fix permission gradlew (APK Android v1.0.4 raté)

### Corrigé
Le build APK Android v1.0.4 a échoué sur CI Linux avec :
```
./gradlew: Permission denied (exit code 126)
```

**Cause** : le projet `/android/` cloné depuis LuxePOS-Web a été copié via
Windows qui ne préserve PAS le bit exécutable Unix sur `gradlew`. Sur Linux
CI (ubuntu-latest), le script n'avait pas droit d'exécution.

**Fix double protection** :
- `git update-index --chmod=+x android/gradlew` → mode 100755 dans l'index
  git, permission propagée à chaque clone (solution permanente)
- Workflow `build-android.yml` : `chmod +x gradlew` avant `./gradlew` (safety
  net si le bit est de nouveau perdu lors d'opérations Windows)

v1.0.4 reste accessible (Win/Mac OK) mais sans APK. v1.0.5 = même contenu
v1.0.4 + APK Android fonctionnel.

## [1.0.4] — 2026-05-26 — APK Android + retrait Lumi (assistant IA)

### Ajouté — APK Android
Le `.tar.gz` présent dans la release v1.0.3 était le format **macOS**
(.app.tar.gz pour auto-updater), pas Android. Maëlle a essayé de l'installer
sur Android → "fichier non supporté".

**Solution** : config Capacitor + workflow CI Android dédié.

- `capacitor.config.json` créé avec `appId: ch.luxepos.lite` (distinct de
  `ch.luxepos.app` du LuxePOS principal — les 2 apks coexistent sur le même
  téléphone).
- `/android/` (projet Gradle Capacitor) cloné depuis LuxePOS-Web, namespace
  et applicationId mis à jour, `app_name` = "LuxePOS Lite".
- `assets/public/luxepos-lite.html` + `index.html` pointent vers le HTML Lite.
- Nouveau workflow `.github/workflows/build-android.yml` :
  * Trigger sur tag `v*` (en parallèle de release.yml Windows/Mac)
  * Setup JDK 17 + Android SDK
  * `./gradlew assembleDebug` → APK debug (~30 MB)
  * Renommé `LuxePOS-Lite-<version>-android.apk` + attaché à la release.

Installation sur Android : activer "Sources inconnues" dans Paramètres, ouvrir
l'APK téléchargé. Pas d'update automatique sur APK debug (release future).

### Retiré — Lumi (assistant IA local)
Devait être retiré dès le scope MVP initial mais Lite v1.0.0-3 contenait
encore la classe complète. Maëlle a re-flaggé :

- `window.lumi = new Lumi()` → `window.lumi = null` (instance désactivée).
  Tous les call sites utilisent `?.method?.()` donc no-op safe.
- `<div id="lumi-bubble">` du DOM retiré (plus de bulle 💎 bottom-right).
- La classe Lumi reste dans le code (~800 lignes, dead code en MVP). Cleanup
  complet planifié en v1.1 simplification.

### Reporté en v1.0.5
- Calculatrice flottante (déplaçable + minimisable badge).

## [1.0.3] — 2026-05-25 — Hotfix synthèse 8 agents audit parallèles

Audit massif par 8 agents indépendants (code, perf, a11y, UX mobile/tablet/desktop,
collisions visuelles, copy FR) a identifié 24 fixes prioritaires. Hotfix applique
les P0 (corruption données) + P1 critiques (UI bloquant) + P2 cohérence devise +
P3 contraste a11y rapide.

### 🔴 P0 — Bugs corruption données (silencieux, destructeurs)
- **`completeSale` décrémente maintenant `stockByLocation[]`** : avant, seul
  `product.stock` était décrémenté → compteurs multi-POS (Atelier/Annemasse/Genève)
  dérivaient à chaque vente. Corrigé en cherchant le slot par `cartItem.locationId`
  ou fallback `product.locationId`.
- **`deleteProduct` détecte enfin les ventes liées** : `(s.items).some(i =>
  i.productId === id)` matchait jamais car `completeSale` push avec `id` (cart
  spread), pas `productId`. Fix : tester `i.id === id || i.productId === id`.
- **`convertReservationToSale` n'écrase plus `currentClient`** : assignait
  l'objet client complet alors que partout ailleurs c'est un string ID → stats
  client jamais MAJ pour ventes issues de réservations. Fix : `res.clientId`.
- **`client.totalSpent` coalesce `|| 0`** + idem `purchaseCount` : empêche NaN
  propagation si client créé/importé sans champs initialisés.

### 🔴 P1 — UI bloquant mobile
- **Bottom nav 6 → 5 items sur mobile** : bouton "Commandes Insta/WhatsApp"
  caché en mobile (`hidden md:flex`), accessible via menu Plus. Évite la
  troncature des labels ("Acc...", "Ven...", "Cat...") à 375px.

### 🔴 P2 — Cohérence devise / copy
- **Atelier "Valeur stock"** : devise dynamique (`settings.currency`) au lieu
  de hardcoder `EUR`. Cohérent avec le reste de l'app (CHF par défaut).
- **Modal "Objectif CA mensuel"** : libellé `Nouvelle cible (€)` → `Nouvelle
  cible (${currency})`.
- **Dropdown paiement** : `💵 Cash` → `💵 Espèces` (anglicisme retiré, valeur
  HTML reste `Espèces` donc rétrocompatible avec les ventes existantes).

### 🔴 P3 — Contraste a11y (WCAG AA)
- **48 helper texts** : `text-xs text-gray-400` → `text-gray-500` (ratio
  passe de 3.55:1 à 5.95:1, conforme WCAG AA pour body text). Ciblé sur
  helper text seulement, préserve les nav décoratifs.

### Reporté en v1.1 (simplification UX guidée par retours testeuses)
- FAB repositionnement avancé, Inventory layout 768px, Modal scroll, TVA
  suisse multi-taux, purge demo data luxe, sidebar labels, role="dialog"
  sur 12 modales, min-h-11 cibles tap, SheetJS lazy-load, virtualisation
  product cards.

## [1.0.2] — 2026-05-25 — URGENT : rollback CSP (Tailwind cassé sur binaire Tauri)

### Corrigé
La CSP ajoutée en v1.0.1 (par mesure sécurité) bloquait l'injection de
styles dynamiques par Tailwind CSS Play CDN dans l'environnement Tauri
WebView2. Résultat : l'app installée affichait du HTML brut sans aucun
style (illisible).

**Cause** : les tests locaux via `http-server` ne reproduisent PAS l'env
Tauri qui applique la CSP. Le bug n'était visible qu'après build + install
du `.exe`.

**Fix** : retour à `csp: null` dans `tauri.conf.json` (état v1.0.0).

### Conservé (les 4 autres fixes sécurité de v1.0.1 restent en place)
- ✅ XSS — `esc()` sur 8 sites (modal variantes, facture)
- ✅ Upload SVG retiré du sélecteur logo
- ✅ Permissions Tauri réduites (fs/process/shell retirés)
- ✅ Validation taille + extension Excel

### À refaire correctement en v1.0.3+
Une CSP compatible Tauri WebView2 + Tailwind dynamique sera ajoutée après
test sur binaire compilé (pas seulement http-server local).

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
