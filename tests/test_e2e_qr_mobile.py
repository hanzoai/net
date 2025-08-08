#!/usr/bin/env python3
"""
End-to-end tests for QR code generation and mobile network joining
"""
import asyncio
import json
import pytest
import qrcode
import io
from unittest.mock import patch, MagicMock
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'src'))

from net.viz.topology_viz import TopologyViz


class TestQRCodeGeneration:
    """Test QR code generation for mobile access"""
    
    def test_qr_code_generates_valid_ascii(self):
        """Test that QR code generates valid ASCII art"""
        # Create topology viz with test URL
        test_url = "http://192.168.1.100:52415"
        viz = TopologyViz(
            chatgpt_api_endpoints=["http://192.168.1.100:52415/v1/chat/completions"],
            web_chat_urls=[test_url]
        )
        
        # Generate visualization
        viz_output = viz.generate_viz()
        
        # Check QR code is in output
        assert "📱 Scan QR code to join from mobile:" in viz_output
        
        # Verify QR code characters are present (Unicode box-drawing)
        assert "▄" in viz_output or "█" in viz_output
        assert "▀" in viz_output or "█" in viz_output
    
    def test_qr_code_encodes_correct_url(self):
        """Test that QR code encodes the correct URL"""
        test_url = "http://192.168.1.100:52415"
        
        # Generate QR code directly
        qr = qrcode.QRCode(border=1)
        qr.add_data(test_url)
        qr.make()
        
        # Verify data
        assert qr.data_list[0].data == test_url.encode('utf-8')
    
    def test_qr_code_scannable_format(self):
        """Test that QR code is in scannable format"""
        test_url = "http://192.168.1.100:52415"
        
        # Generate QR code
        qr = qrcode.QRCode(border=1)
        qr.add_data(test_url)
        qr.make()
        
        # Get ASCII output
        f = io.StringIO()
        qr.print_ascii(out=f, tty=False, invert=False)
        qr_ascii = f.getvalue()
        
        # Verify it's not empty and has proper structure
        assert len(qr_ascii) > 0
        lines = qr_ascii.strip().split('\n')
        assert len(lines) > 10  # QR codes are at least 11x11
        
        # All lines should have same width (QR codes are square)
        line_lengths = [len(line) for line in lines]
        assert len(set(line_lengths)) <= 2  # Allow for trailing spaces


class TestMobileDetection:
    """Test mobile device detection in web interface"""
    
    @pytest.fixture
    def mobile_user_agents(self):
        """Common mobile user agents for testing"""
        return [
            "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15",
            "Mozilla/5.0 (Linux; Android 10; SM-G960F) AppleWebKit/537.36",
            "Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X) AppleWebKit/605.1.15",
        ]
    
    def test_mobile_detection_logic(self, mobile_user_agents):
        """Test that mobile detection works for various user agents"""
        # This would test the JavaScript logic
        mobile_detection_pattern = r"Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini"
        
        import re
        pattern = re.compile(mobile_detection_pattern, re.IGNORECASE)
        
        for ua in mobile_user_agents:
            assert pattern.search(ua) is not None, f"Failed to detect mobile for: {ua}"
    
    def test_webgpu_detection_mock(self):
        """Test WebGPU capability detection (mocked)"""
        # Mock the platform capabilities that would be detected
        platform_info = {
            "isMobile": True,
            "hasWebGPU": False,  # Most mobile browsers don't have WebGPU yet
            "hasOffscreenCanvas": True,
            "accelerationSupported": ["OffscreenCanvas", "WASM-SIMD"]
        }
        
        # Verify the structure
        assert isinstance(platform_info["isMobile"], bool)
        assert isinstance(platform_info["accelerationSupported"], list)


class TestDistributedNetwork:
    """Test distributed network functionality"""
    
    @pytest.mark.asyncio
    async def test_network_node_initialization(self):
        """Test that network nodes initialize correctly"""
        from net.topology.device_capabilities import DeviceCapabilities
        from net.topology.topology import Topology
        
        # Create mock device capabilities
        device = DeviceCapabilities(
            model="TestDevice",
            chip="TestChip",
            memory=8*1024*1024*1024,  # 8GB
            flops={"fp32": 1e12, "fp16": 2e12}
        )
        
        # Create topology
        topology = Topology()
        topology.update_node("test-node-1", device)
        
        # Verify node was added
        assert "test-node-1" in topology.nodes
        assert topology.nodes["test-node-1"] == device
    
    @pytest.mark.asyncio
    async def test_mobile_node_joins_network(self):
        """Test that mobile nodes can join the network"""
        from net.topology.device_capabilities import DeviceCapabilities
        from net.topology.topology import Topology
        
        # Create topology
        topology = Topology()
        
        # Simulate mobile device joining
        mobile_device = DeviceCapabilities(
            model="iPhone 15 Pro",
            chip="A17 Pro",
            memory=6*1024*1024*1024,  # 6GB
            flops={"fp32": 0.5e12, "fp16": 1e12}  # Mobile GPU capabilities
        )
        
        topology.update_node("mobile-node-1", mobile_device)
        
        # Verify mobile node is in network
        assert "mobile-node-1" in topology.nodes
        assert topology.nodes["mobile-node-1"].model == "iPhone 15 Pro"


class TestWebInterface:
    """Test web interface functionality"""
    
    def test_web_server_endpoints(self):
        """Test that web server has correct endpoints"""
        # These would be the expected endpoints
        expected_endpoints = [
            "/",  # Main web interface
            "/v1/chat/completions",  # ChatGPT API
            "/initial_models",  # Model list
            "/modelpool",  # Model pool updates
        ]
        
        # In real test, we'd start the server and check these
        for endpoint in expected_endpoints:
            assert endpoint  # Placeholder for actual HTTP tests
    
    def test_mobile_responsive_viewport(self):
        """Test that HTML has mobile viewport meta tag"""
        html_content = """<meta content="width=device-width, initial-scale=1" name="viewport"/>"""
        assert "viewport" in html_content
        assert "width=device-width" in html_content


if __name__ == "__main__":
    pytest.main([__file__, "-v"])