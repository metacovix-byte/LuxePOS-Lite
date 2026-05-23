// LuxePOS Lite — Configuration Playwright
// Port 8766 (LuxePOS principal utilise 8765, donc pas de conflit local)

const { defineConfig } = require('@playwright/test');

module.exports = defineConfig({
    testDir: './tests',
    timeout: 30 * 1000,
    expect: { timeout: 5000 },
    fullyParallel: false,
    workers: 1,
    retries: 0,
    reporter: [['list'], ['html', { open: 'never' }]],
    use: {
        baseURL: 'http://localhost:8766',
        trace: 'retain-on-failure',
        screenshot: 'only-on-failure'
    },
    projects: [
        {
            name: 'chromium',
            use: {
                browserName: 'chromium',
                viewport: { width: 1280, height: 820 }
            }
        }
    ]
});
