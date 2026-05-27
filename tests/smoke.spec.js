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
});
