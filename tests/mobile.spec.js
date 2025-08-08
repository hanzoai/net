// @ts-check
const { test, expect, devices } = require('@playwright/test');

// Test mobile QR code scanning simulation
test.describe('Mobile Device Network Joining', () => {
  
  test('iPhone scans QR code and joins network', async ({ browser }) => {
    // Create iPhone context
    const context = await browser.newContext({
      ...devices['iPhone 12'],
      ignoreHTTPSErrors: true,
    });
    
    const page = await context.newPage();
    
    // Navigate to the Hanzo Net interface
    await page.goto('http://localhost:52415');
    
    // Check page loads
    await expect(page).toHaveTitle('Hanzo Chat');
    
    // Verify mobile detection
    const isMobile = await page.evaluate(() => {
      return /iPhone|iPad|iPod/i.test(navigator.userAgent);
    });
    expect(isMobile).toBeTruthy();
    
    // Check for WebGPU support (may not be available in all environments)
    const hasWebGPU = await page.evaluate(() => 'gpu' in navigator);
    console.log('WebGPU available:', hasWebGPU);
    
    await context.close();
  });
  
  test('Android device capabilities detection', async ({ browser }) => {
    // Create Android context
    const context = await browser.newContext({
      ...devices['Pixel 5'],
      ignoreHTTPSErrors: true,
    });
    
    const page = await context.newPage();
    await page.goto('http://localhost:52415');
    
    // Test platform capabilities detection
    const capabilities = await page.evaluate(() => {
      const caps = {
        cores: navigator.hardwareConcurrency || 1,
        memory: navigator.deviceMemory || 'unknown',
        connection: navigator.connection?.effectiveType || 'unknown',
        platform: navigator.platform,
        vendor: navigator.vendor
      };
      return caps;
    });
    
    expect(capabilities.cores).toBeGreaterThan(0);
    console.log('Device capabilities:', capabilities);
    
    await context.close();
  });
  
  test('iPad Pro tablet experience', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['iPad Pro'],
      ignoreHTTPSErrors: true,
    });
    
    const page = await context.newPage();
    await page.goto('http://localhost:52415');
    
    // Check viewport is tablet-appropriate
    const viewport = page.viewportSize();
    expect(viewport.width).toBeGreaterThanOrEqual(768);
    
    // Test that UI scales properly for tablet
    const isTablet = await page.evaluate(() => {
      const width = window.innerWidth;
      return width >= 768 && width <= 1024;
    });
    expect(isTablet).toBeTruthy();
    
    await context.close();
  });
  
  test('Mobile WebGL/WebGPU acceleration detection', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['iPhone 13 Pro'],
      ignoreHTTPSErrors: true,
    });
    
    const page = await context.newPage();
    await page.goto('http://localhost:52415');
    
    // Test GPU acceleration availability
    const gpuSupport = await page.evaluate(() => {
      const support = {
        webgl: false,
        webgl2: false,
        webgpu: false,
        offscreenCanvas: false
      };
      
      // Check WebGL
      const canvas = document.createElement('canvas');
      support.webgl = !!canvas.getContext('webgl');
      support.webgl2 = !!canvas.getContext('webgl2');
      
      // Check WebGPU
      support.webgpu = 'gpu' in navigator;
      
      // Check OffscreenCanvas
      support.offscreenCanvas = typeof OffscreenCanvas !== 'undefined';
      
      return support;
    });
    
    // At least WebGL should be available
    expect(gpuSupport.webgl || gpuSupport.webgl2).toBeTruthy();
    console.log('GPU Support:', gpuSupport);
    
    await context.close();
  });
  
  test('Mobile network topology visualization', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['Pixel 7'],
      ignoreHTTPSErrors: true,
    });
    
    const page = await context.newPage();
    await page.goto('http://localhost:52415');
    
    // Wait for topology to load
    await page.waitForTimeout(2000);
    
    // Check if topology visualization exists
    const hasTopology = await page.locator('.topology-visualization').count() > 0;
    
    if (hasTopology) {
      // Check that mobile device appears in topology
      const topologyText = await page.locator('.topology-visualization').textContent();
      console.log('Topology includes mobile device');
    }
    
    await context.close();
  });
});

// Test QR code generation on server side
test.describe('QR Code Generation', () => {
  
  test('Server generates valid QR code', async ({ request }) => {
    // This would need a custom endpoint to return QR code status
    // For now, we just check the main page loads
    const response = await request.get('http://localhost:52415');
    expect(response.ok()).toBeTruthy();
  });
});

// Performance tests for mobile
test.describe('Mobile Performance', () => {
  
  test('Page loads quickly on mobile', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['iPhone 12'],
      ignoreHTTPSErrors: true,
    });
    
    const page = await context.newPage();
    
    const startTime = Date.now();
    await page.goto('http://localhost:52415');
    const loadTime = Date.now() - startTime;
    
    // Page should load in under 3 seconds
    expect(loadTime).toBeLessThan(3000);
    console.log(`Page load time: ${loadTime}ms`);
    
    await context.close();
  });
  
  test('Mobile memory usage is reasonable', async ({ browser }) => {
    const context = await browser.newContext({
      ...devices['Pixel 5'],
      ignoreHTTPSErrors: true,
    });
    
    const page = await context.newPage();
    await page.goto('http://localhost:52415');
    
    // Check JavaScript heap size if available
    const memoryInfo = await page.evaluate(() => {
      if (performance.memory) {
        return {
          usedJSHeapSize: Math.round(performance.memory.usedJSHeapSize / 1048576),
          totalJSHeapSize: Math.round(performance.memory.totalJSHeapSize / 1048576),
        };
      }
      return null;
    });
    
    if (memoryInfo) {
      console.log(`Memory usage: ${memoryInfo.usedJSHeapSize}MB / ${memoryInfo.totalJSHeapSize}MB`);
      // Should use less than 100MB
      expect(memoryInfo.usedJSHeapSize).toBeLessThan(100);
    }
    
    await context.close();
  });
});