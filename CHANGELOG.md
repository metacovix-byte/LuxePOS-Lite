# Changelog — LuxePOS Lite

Toutes les versions notables de LuxePOS Lite sont documentées ici.

Format inspiré de [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/).
Versioning : [SemVer](https://semver.org/spec/v2.0.0.html).

## [1.0.7] — 2026-05-29 — Page Feedback fiable pour les testeuses

### Corrigé — Dialog « Signaler un problème » cassé
Le dialog était accessible depuis le menu Plus mais ne marchait pas pour les testeuses :

- **Version hardcodée fausse** : affichait `LuxePOS v5.8 (Build 2026-04-26)` au lieu
  de `LuxePOS Lite v1.0.7` → impossible de savoir quelle build est concernée par
  un bug report.
- **Email destinataire fictif** : `mailto:support@luxepos.local` → adresse qui
  n'existe pas, l'email atterrissait dans le néant. Maintenant pointe vers
  `metacovix@gmail.com` (boîte de support réelle).
- **Pas de détection plateforme / OS** : impossible de savoir si le bug est
  sur Windows Tauri, macOS Tauri, Android Capacitor ou Web. Maintenant
  le contexte inclut `plateforme` (Tauri/Capacitor/Web) + `os` (Windows/macOS/Android/iOS/Linux).
- **Nom de la boutique** ajouté dans le contexte pour identifier la testeuse
  sans qu'elle ait à se nommer.

### Ajouté
- **Bouton « Copier seulement »** : pour les testeuses dont le client email
  n'est pas configuré (mailto: échoue silencieusement). Copie le message
  complet dans le presse-papier — collable dans WhatsApp, Insta DM, Discord,
  ou n'importe quel webmail.
- **Fallback presse-papier robuste** : `navigator.clipboard.writeText()` puis
  fallback `textarea + execCommand('copy')` pour les vieux WebViews Android.
- **Toast plus clair** : indique où coller le message si l'email ne s'ouvre pas.

### Corrigé — Audit adversarial (28 agents, 22 findings confirmés)
Un audit multi-agent a révélé que le `mailto:` était **cassé sur les plateformes
natives** — exactement celles qu'utilisent les testeuses :

- **P1 — `window.location.href = mailto` ne marche PAS dans la WebView Tauri**
  (Windows/macOS). L'assignation échoue silencieusement, aucun client email ne
  s'ouvre, mais un faux toast « Email ouvert » s'affichait. → Désormais routé via
  le **plugin shell Tauri** (`invoke('plugin:shell|open')`, autorisé par la
  capability `shell:allow-open`), qui ouvre réellement le client email de l'OS.
- **P1 — sur Capacitor Android**, `window.location.href = mailto` pouvait
  naviguer la WebView hors de l'app. → Route via `@capacitor/app` (App.openUrl)
  si présent, sinon `window.open('_system')`.
- **P1 — faux toast de succès** : le toast affichait toujours « Email ouvert »
  même en cas d'échec. → `_openExternal()` renvoie un booléen ; le toast est
  maintenant **honnête** (succès réel vs « aucun client email détecté, message
  copié, collez-le dans WhatsApp »).
- **P1 — fuite mémoire** : le listener `keydown` (Escape) n'était jamais retiré,
  s'accumulant à chaque ouverture. → `_closeFeedbackDialog()` centralise le
  cleanup (retrait listener + remove overlay), utilisé par tous les chemins de
  fermeture (X, Annuler, Escape, clic overlay, après envoi).
- **P2 — destinataire invisible** : l'email n'apparaissait que dans le bloc
  technique replié. → Ligne « 📬 Destinataire : metacovix@gmail.com » visible
  sous les boutons, avant l'envoi.
- **P2 — détection plateforme dupliquée/fragile** inline. → Réutilise les
  helpers éprouvés `window.store._isTauri()` / `_isCapacitor()`.
- **P2 — label trompeur** « Ouvrir mon email » → « Envoyer par email » + tooltip
  explicatif (fallback presse-papier si pas de client).
- **P2 — toast trop court** (6 s → 8-9 s) + instruction de collage explicite.
- **P3 — perf** : `stateSizeKB` réutilise la sérialisation en cache
  (`store._lastSerialized`) au lieu de re-`JSON.stringify` tout le state à chaque
  ouverture.
- **P3 — mailto trop long** : corps mailto en JSON compact (le presse-papier
  garde la version lisible indentée).
- **P3 — email en dur** centralisé dans `APP_CONFIG.SUPPORT_EMAIL`.
- **P3 — clarté RGPD** : le bloc technique précise « reste dans votre message,
  jamais partagé ailleurs ».

2 findings rejetés (faux positifs) : pas de fuite du state complet (seul un objet
métadonnées est envoyé), pas de XSS (escapeHtml correctement appliqué sur le `<pre>`).

### Tests
- Test 008 : version dynamique dans le contexte, absence de `v5.8`, destinataire
  visible à l'écran, `SUPPORT_EMAIL` depuis APP_CONFIG, absence de
  `support@luxepos.local`.
- Test 009 : ouverture/fermeture ×3 → listener Escape nettoyé (pas de fuite).
- Test 010 : `_openExternal` route via `plugin:shell|open` quand Tauri est simulé
  (et non `window.location.href`).

## [1.0.6] — 2026-05-28 — Calculatrice flottante (draggable + minimisable)

### Ajouté — Calculatrice flottante
Demande Mounir : la calculatrice était une modale plein-écran bloquante,
impossible de voir l'inventaire en même temps. Maintenant :

- **Panneau flottant** au lieu de modale : reste visible pendant qu'on
  travaille sur d'autres pages (inventaire, POS, clients).
- **Draggable** : barre titre déplaçable à la souris ou au doigt (touch).
  Clamp aux bords du viewport pour ne jamais sortir de l'écran.
- **Minimisable** : bouton `−` réduit la calculatrice en **badge compact**
  (icône + dernière valeur) en bas à droite. Un clic restaure le panneau
  à sa dernière position.
- **Bouton %** ajouté : calcule les pourcentages contextuellement.
  `150 + 30% =` donne `195` (30% de 150 = 45). Utile pour commissions
  dépôt-vente (ex: 30% Annemasse, 25% Salon Genève).
- **Arrondi propre** : `Math.round(result * 1e10) / 1e10` évite les
  artifacts floating-point (`0.1 + 0.2 = 0.3`, pas `0.30000000000000004`).
- Tip contextuel et Command Palette mis à jour.

### Technique
- CSS : `#calc-floating` (fixed, z-9998, 340px, shadow, spring-in anim)
  + `#calc-badge` (fixed, pill, gradient indigo/violet).
- JS : `_calcInitDrag(panel)` mouse+touch, `_calcMinimize()`,
  `_calcRestore()`, `_calcClose()`, `calcInput('%')`.
- Position mémorisée dans `this._calcPos` entre minimize/restore.
- Mobile : largeur 100% - 16px sous 480px.

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
