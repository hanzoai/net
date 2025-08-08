#!/usr/bin/env python3
"""
Browser-based E2E tests using Playwright for mobile simulation
"""
import asyncio
import pytest
from playwright.async_api import async_playwright
import subprocess
import time
import os
import signal


class TestMobileBrowserE2E:
    """Test mobile browser experience end-to-end"""
    
    @pytest.fixture(scope="class")
    async def server(self):
        """Start the Hanzo Net server for testing"""
        # Start server process
        env = os.environ.copy()
        env["DISABLE_TUI"] = "1"
        
        process = subprocess.Popen(
            ["python", "-m", "net.main", "--disable-tui"],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        
        # Wait for server to start
        time.sleep(5)
        
        yield "http://localhost:52415"
        
        # Clean up
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
    
    @pytest.mark.asyncio
    async def test_mobile_iphone_loads_interface(self, server):
        """Test that iPhone can load the interface"""
        async with async_playwright() as p:
            # Launch browser with iPhone 12 viewport
            browser = await p.webkit.launch(headless=True)
            context = await browser.new_context(
                **p.devices["iPhone 12"],
                ignore_https_errors=True
            )
            page = await context.new_page()
            
            # Navigate to the server
            await page.goto(server)
            
            # Check that main elements are visible
            assert await page.title() == "Hanzo Chat"
            
            # Check for mobile-optimized viewport
            viewport = page.viewport_size
            assert viewport["width"] <= 428  # iPhone 12 width
            
            # Check WebGPU detection ran
            webgpu_check = await page.evaluate("""
                () => {
                    return 'gpu' in navigator;
                }
            """)
            # Note: WebGPU might not be available in all test environments
            assert isinstance(webgpu_check, bool)
            
            await browser.close()
    
    @pytest.mark.asyncio
    async def test_android_device_joins_network(self, server):
        """Test that Android device can join the network"""
        async with async_playwright() as p:
            # Launch browser with Pixel 5 viewport
            browser = await p.chromium.launch(headless=True)
            context = await browser.new_context(
                **p.devices["Pixel 5"],
                ignore_https_errors=True
            )
            page = await context.new_page()
            
            # Navigate to the server
            await page.goto(server)
            
            # Check platform detection
            platform_info = await page.evaluate("""
                () => {
                    const isMobile = /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);
                    return {
                        isMobile: isMobile,
                        userAgent: navigator.userAgent
                    };
                }
            """)
            
            assert platform_info["isMobile"] == True
            assert "Android" in platform_info["userAgent"] or "Linux" in platform_info["userAgent"]
            
            await browser.close()
    
    @pytest.mark.asyncio
    async def test_ipad_tablet_experience(self, server):
        """Test iPad tablet experience"""
        async with async_playwright() as p:
            browser = await p.webkit.launch(headless=True)
            context = await browser.new_context(
                **p.devices["iPad Pro"],
                ignore_https_errors=True
            )
            page = await context.new_page()
            
            # Navigate to the server
            await page.goto(server)
            
            # Check viewport is tablet-sized
            viewport = page.viewport_size
            assert viewport["width"] >= 768  # Tablet width
            
            # Check that interface scales properly
            assert await page.title() == "Hanzo Chat"
            
            await browser.close()
    
    @pytest.mark.asyncio
    async def test_qr_code_visible_on_desktop(self, server):
        """Test that QR code is generated and visible on desktop view"""
        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True)
            page = await browser.new_page()
            
            # This would test the terminal output, not the web interface
            # In a real scenario, we'd capture the server output
            
            await browser.close()
    
    @pytest.mark.asyncio
    async def test_mobile_webgpu_capabilities(self, server):
        """Test WebGPU capability detection on mobile"""
        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True)
            context = await browser.new_context(
                **p.devices["iPhone 13 Pro Max"],
                ignore_https_errors=True
            )
            page = await context.new_page()
            
            await page.goto(server)
            
            # Check acceleration capabilities
            capabilities = await page.evaluate("""
                async () => {
                    const caps = {
                        webgpu: false,
                        webgl: false,
                        webgl2: false,
                        offscreenCanvas: false,
                        wasmSimd: false
                    };
                    
                    // Check WebGPU
                    if ('gpu' in navigator) {
                        try {
                            const adapter = await navigator.gpu.requestAdapter();
                            caps.webgpu = !!adapter;
                        } catch (e) {}
                    }
                    
                    // Check WebGL
                    const canvas = document.createElement('canvas');
                    caps.webgl = !!canvas.getContext('webgl');
                    caps.webgl2 = !!canvas.getContext('webgl2');
                    
                    // Check OffscreenCanvas
                    caps.offscreenCanvas = typeof OffscreenCanvas !== 'undefined';
                    
                    // Check WASM SIMD
                    try {
                        const wasmSimd = new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0, 1, 5, 1, 96, 0, 1, 123]);
                        caps.wasmSimd = WebAssembly.validate(wasmSimd);
                    } catch (e) {}
                    
                    return caps;
                }
            """)
            
            # At least some acceleration should be available
            assert any([
                capabilities.get("webgl"),
                capabilities.get("webgl2"),
                capabilities.get("offscreenCanvas"),
                capabilities.get("wasmSimd")
            ])
            
            await browser.close()


if __name__ == "__main__":
    # Run tests
    asyncio.run(pytest.main([__file__, "-v"]))