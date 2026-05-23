# LuxePOS Lite

**MVP simplifié pour artisanes bijoutières — version testeuses early access**

Outil de saisie & stock dérivé du projet [LuxePOS](https://github.com/metacovix-byte/LuxePOS-Web) (single-file HTML + Tauri + Capacitor). Conçu pour 3 testeuses qui veulent un outil **rapide à prendre en main**, sans la complexité de la version complète.

## ✨ Fonctions v1.0

- 📦 **Saisie produits** : nom, référence, prix, coût, stock, catégorie, emplacement
- 📸 **Photos haute qualité** (~200 KB, max 1200×1200) : webcam, import dossier, paste Ctrl+V
- 📊 **Import Excel intelligent** : 11 feuilles, mapping auto refs / prix / emplacements / ventes
- 🏬 **Multi-POS** : 3 emplacements (Atelier CHF · Annemasse EUR · Salon Genève CHF) avec commissions
- ✅ **Toggle Vendu/Pas vendu** : 1 click, date + à qui (pas de panier, pas de paiement)
- 👤 **Clients** : annuaire léger pour retrouver les acheteuses
- 📋 **Historique d'activité** : qui a fait quoi quand (3 testeuses = trace utile)
- ☁️ **Backup OneDrive chiffré** : AES-GCM 256, clé locale jamais partagée
- 📤 **Export Excel** : pour comptable / backup manuel
- 🔌 **Mode offline** : marche sans réseau, sync à la reconnexion
- 💬 **Page Feedback** : 5-8 questions pour remonter avis/bugs (via Formspree)

## 🚫 Hors-périmètre v1.0 (potentiellement v2+)

POS / caisse / panier · WhatsApp / Instagram pipeline · SAV / Réparations · Lumi (IA locale) · BOM / Composants · Dashboard analytics complet · Multi-thèmes.

## 📥 Installation

Téléchargez la dernière version : [GitHub Releases](https://github.com/metacovix-byte/LuxePOS-Lite/releases/latest)

- **Windows** : `LuxePOS_Lite_1.0.0_x64-setup.exe` (~5 MB, installeur NSIS signé)
- **macOS** : `LuxePOS_Lite_1.0.0_universal.dmg` (~7 MB, Intel + Apple Silicon)
- **Android** : `LuxePOS-Lite.apk` (~30 MB, avec scanner code-barres ML Kit)

Auto-updates : l'app détecte les nouvelles versions au démarrage et propose l'update.

## 🛠️ Dev local

```bash
npm install
npm run inline-vendors     # inline Tailwind/Lucide/SheetJS dans le HTML
npx http-server -p 8766    # serveur local (port 8766 distinct de LuxePOS)
# Puis ouvrir : http://localhost:8766/luxepos-lite.html
```

## 🔄 Relation avec LuxePOS principal

LuxePOS Lite est un **fork délibérément simplifié** du codebase principal. Quand un bug critique est corrigé dans LuxePOS, il est porté manuellement vers Lite si pertinent. La direction inverse (Lite → LuxePOS) est rare car Lite n'innove pas, il simplifie.

Plus tard, quand Lite sera stabilisé, les modules communs (Store, Excel parser, OneDrive sync, photos) pourront être extraits dans une lib partagée.

## 📄 Licence

MIT. Voir [LICENSE](LICENSE).
