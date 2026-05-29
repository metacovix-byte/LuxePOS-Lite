// LuxePOS Lite — smoke tests minimal pour MVP
// Le grand catalogue de tests vit dans LuxePOS-Web. Ici on garde l'essentiel.

const { test, expect } = require('@playwright/test');
const BASE_URL = 'http://localhost:8766/luxepos-lite.html';

test.describe('LuxePOS Lite — Smoke tests v1.0', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL, { waitUntil: 'load' });
        await page.waitForTimeout(1000);
        await page.waitForFunction(() => window.store && window.ui, { timeout: 15000 });
    });

    test('001. Page charge sans erreur JS', async ({ page }) => {
        const errors = [];
        page.on('pageerror', e => errors.push(e.message));
        await page.waitForTimeout(500);
        // Tolère les erreurs liées aux modules retirés (modal Lumi absente, etc.)
        // mais aucune erreur de boot critique
        const critical = errors.filter(e => /TypeError|ReferenceError/.test(e) && !/lumi|whatsapp|sav/i.test(e));
        expect(critical.length).toBe(0);
    });

    test('002. APP_CONFIG version correcte', async ({ page }) => {
        const cfg = await page.evaluate(() => ({
            version: window.APP_CONFIG?.VERSION,
            edition: window.APP_CONFIG?.EDITION
        }));
        expect(cfg.version).toMatch(/^\d+\.\d+\.\d+/);
        expect(cfg.edition).toBe('Lite');
    });

    test('003. Store initialisé avec 3 emplacements (multi-POS)', async ({ page }) => {
        const locations = await page.evaluate(() => window.store.state.locations || []);
        expect(locations.length).toBeGreaterThanOrEqual(3);
        expect(locations.find(l => l.id === 'loc_atelier')).toBeTruthy();
        expect(locations.find(l => l.id === 'loc_annemasse')).toBeTruthy();
        expect(locations.find(l => l.id === 'loc_geneve')).toBeTruthy();
    });

    test('004. Devise CHF par défaut + formatCurrency apostrophe milliers', async ({ page }) => {
        const result = await page.evaluate(() => ({
            currency: window.store.state.settings?.currency,
            formatted: window.formatCurrency(1234.5, { currency: 'CHF', decimals: 2 })
        }));
        expect(result.currency).toBe('CHF');
        expect(result.formatted).toContain("'");
    });

    test('005. XLSX (SheetJS) chargé en local — offline-first', async ({ page }) => {
        const result = await page.evaluate(() => ({
            xlsxLoaded: typeof window.XLSX !== 'undefined' && typeof window.XLSX.read === 'function',
            cdnRefCount: Array.from(document.querySelectorAll('script[src]'))
                .filter(s => /xlsx|chart\.js|confetti|tilt/.test(s.src) && /cdn|jsdelivr|unpkg/.test(s.src))
                .length
        }));
        expect(result.xlsxLoaded).toBe(true);
        expect(result.cdnRefCount).toBe(0);
    });

    test('006. Calculatrice flottante : ouvre, calcule, minimise, restaure, ferme', async ({ page }) => {
        // Ouvrir la calculatrice
        await page.evaluate(() => window.ui.openCalculator());
        await page.waitForTimeout(300);

        // Vérifier que le panneau flottant existe (pas une modale plein-écran)
        const isFloating = await page.evaluate(() => {
            const el = document.getElementById('calc-floating');
            return el !== null && el.style.position !== 'static';
        });
        expect(isFloating).toBe(true);

        // Tester un calcul : 150 + 30% = 195 (commission 30%)
        await page.evaluate(() => {
            const ui = window.ui;
            ui.calcInput('1'); ui.calcInput('5'); ui.calcInput('0');
            ui.calcInput('+');
            ui.calcInput('3'); ui.calcInput('0');
            ui.calcInput('%'); // 30% de 150 = 45
            ui.calcInput('='); // 150 + 45 = 195
        });
        const result = await page.evaluate(() => document.getElementById('calc-display')?.textContent);
        expect(result).toBe('195');

        // Minimiser → badge apparaît, panneau disparaît
        await page.evaluate(() => window.ui._calcMinimize());
        await page.waitForTimeout(200);
        const afterMinimize = await page.evaluate(() => ({
            hasBadge: document.getElementById('calc-badge') !== null,
            hasPanel: document.getElementById('calc-floating') !== null
        }));
        expect(afterMinimize.hasBadge).toBe(true);
        expect(afterMinimize.hasPanel).toBe(false);

        // Restaurer → panneau revient, badge disparaît
        await page.evaluate(() => window.ui._calcRestore());
        await page.waitForTimeout(300);
        const afterRestore = await page.evaluate(() => ({
            hasBadge: document.getElementById('calc-badge') !== null,
            hasPanel: document.getElementById('calc-floating') !== null,
            value: document.getElementById('calc-display')?.textContent
        }));
        expect(afterRestore.hasBadge).toBe(false);
        expect(afterRestore.hasPanel).toBe(true);
        expect(afterRestore.value).toBe('195'); // valeur préservée

        // Fermer → tout disparaît
        await page.evaluate(() => window.ui._calcClose());
        await page.waitForTimeout(250);
        const afterClose = await page.evaluate(() => ({
            hasBadge: document.getElementById('calc-badge') !== null,
            hasPanel: document.getElementById('calc-floating') !== null
        }));
        expect(afterClose.hasBadge).toBe(false);
        expect(afterClose.hasPanel).toBe(false);
    });

    test('007. Calculatrice : arrondi flottant propre (0.1 + 0.2 = 0.3)', async ({ page }) => {
        await page.evaluate(() => {
            const ui = window.ui;
            ui.openCalculator();
            ui.calcInput('0'); ui.calcInput('.'); ui.calcInput('1');
            ui.calcInput('+');
            ui.calcInput('0'); ui.calcInput('.'); ui.calcInput('2');
            ui.calcInput('=');
        });
        const result = await page.evaluate(() => document.getElementById('calc-display')?.textContent);
        expect(result).toBe('0.3'); // NOT 0.30000000000000004
    });

    test('008. Feedback dialog : ouvre, contexte dynamique, email config, destinataire visible', async ({ page }) => {
        await page.evaluate(() => window.ui.showFeedbackDialog());
        await page.waitForTimeout(200);

        const overlayExists = await page.evaluate(() => !!document.getElementById('feedback-overlay'));
        expect(overlayExists).toBe(true);

        // Contexte technique : version dynamique (pas l'ancienne v5.8 hardcodée), champs v1.0.7
        const ctxStr = await page.evaluate(() => document.querySelector('#feedback-overlay pre')?.textContent || '');
        expect(ctxStr).toContain('1.0');
        expect(ctxStr).not.toContain('v5.8');
        expect(ctxStr).toMatch(/plateforme/i);
        expect(ctxStr).toMatch(/\bos\b/i);

        // Destinataire visible À L'ÉCRAN avant l'envoi (finding UX P2)
        const dialogText = await page.evaluate(() => document.getElementById('feedback-overlay')?.textContent || '');
        expect(dialogText).toContain('metacovix@gmail.com');
        expect(dialogText).toContain('Destinataire');

        // L'email destinataire vient de APP_CONFIG (plus de hardcode épars)
        const cfgEmail = await page.evaluate(() => window.APP_CONFIG?.SUPPORT_EMAIL);
        expect(cfgEmail).toBe('metacovix@gmail.com');

        // _sendFeedback ne doit plus contenir l'ancien email fictif
        const sendSrc = await page.evaluate(() => window.ui._sendFeedback.toString());
        expect(sendSrc).not.toContain('support@luxepos.local');

        await page.evaluate(() => window.ui._closeFeedbackDialog());
    });

    test('009. Feedback : listener Escape nettoyé (pas de fuite), fermeture propre', async ({ page }) => {
        // Ouvre/ferme 3× : le handler Escape ne doit pas s'accumuler
        const leak = await page.evaluate(async () => {
            for (let i = 0; i < 3; i++) {
                window.ui.showFeedbackDialog();
                await new Promise(r => setTimeout(r, 50));
                window.ui._closeFeedbackDialog();
                await new Promise(r => setTimeout(r, 50));
            }
            // Après fermeture, _feedbackEsc doit être null (listener retiré)
            return {
                escIsNull: window.ui._feedbackEsc === null || window.ui._feedbackEsc === undefined,
                overlayGone: document.getElementById('feedback-overlay') === null
            };
        });
        expect(leak.escIsNull).toBe(true);
        expect(leak.overlayGone).toBe(true);
    });

    test('010. Feedback : _openExternal route via shell sur Tauri (pas window.location.href)', async ({ page }) => {
        // Simule un environnement Tauri et vérifie que _openExternal invoque plugin:shell|open
        const result = await page.evaluate(async () => {
            const calls = { invokeCmd: null, invokeArgs: null, locationChanged: false };
            // Stub Tauri internals
            const origTauri = window.__TAURI_INTERNALS__;
            const origIsTauri = window.store._isTauri;
            window.__TAURI_INTERNALS__ = {
                invoke: (cmd, args) => { calls.invokeCmd = cmd; calls.invokeArgs = args; return Promise.resolve(); }
            };
            window.store._isTauri = () => true;
            window.ui._feedbackMeta = { isTauri: true, isCapacitor: false, supportEmail: 'metacovix@gmail.com' };
            try {
                const ok = await window.ui._openExternal('mailto:metacovix@gmail.com?subject=test');
                calls.ok = ok;
            } finally {
                window.__TAURI_INTERNALS__ = origTauri;
                window.store._isTauri = origIsTauri;
                window.ui._feedbackMeta = null;
            }
            return calls;
        });
        expect(result.ok).toBe(true);
        expect(result.invokeCmd).toBe('plugin:shell|open');
        expect(result.invokeArgs?.path).toContain('mailto:metacovix@gmail.com');
    });
});
