import sys
import platform
import subprocess

from setuptools import find_packages, setup

# Base requirements for all platforms
install_requires = [
  "aiohttp==3.10.11",
  "aiohttp_cors==0.7.0",
  "aiofiles==24.1.0",
  "grpcio==1.70.0",
  "grpcio-tools==1.70.0",
  "Jinja2==3.1.4",
  "numpy==2.0.0",
  "nuitka==2.5.1",
  "nvidia-ml-py==12.560.30",
  "opencv-python==4.10.0.84",
  "pillow==10.4.0",
  "prometheus-client==0.20.0",
  "protobuf==5.28.1",
  "psutil==6.0.0",
  "pyamdgpuinfo==2.1.6;platform_system=='Linux'",
  "pydantic==2.9.2",
  "requests==2.32.3",
  "rich==13.7.1",
  "scapy==2.6.1",
  "tqdm==4.66.4",
  "transformers==4.46.3",
  "uuid==1.30",
  "uvloop==0.21.0",
  "tinygrad>=0.10.0",  # Using PyPI version instead of git
  "tokenizers>=0.20.0",  # Required for transformers
  "huggingface-hub>=0.20.0",  # Required for model downloads
  "qrcode[pil]>=7.0.0",  # For QR code generation
]

extras_require = {
  "formatting": ["yapf==0.40.2",],
  "apple_silicon": [
    "mlx>=0.22.0",
    "mlx-lm>=0.21.1",
  ],
  "windows": ["pywin32==308",],
  "nvidia-gpu": ["nvidia-ml-py==12.560.30",],
  "amd-gpu": ["pyrsmi==0.2.0"],
}

# Handle MLX installation based on platform
if sys.platform.startswith("darwin"):
  if platform.machine() == "arm64":
    # Apple Silicon - add MLX support automatically
    install_requires.extend(extras_require["apple_silicon"])
    print("✅ Apple Silicon detected - MLX support enabled")
  else:
    # Intel Mac - MLX not available, inform user
    print("ℹ️  Intel Mac detected - MLX not available (Apple Silicon only)")
    print("   Hanzo Net will work with CPU and other acceleration backends")

# Check if running Windows
if sys.platform.startswith("win32"):
  install_requires.extend(extras_require["windows"])


def _add_gpu_requires():
  global install_requires
  # Add Nvidia-GPU
  try:
    out = subprocess.run(['nvidia-smi', '--query-gpu=name', '--format=csv,noheader'], shell=True, text=True, capture_output=True, check=False)
    if out.returncode == 0:
      install_requires.extend(extras_require["nvidia-gpu"])
  except subprocess.CalledProcessError:
    pass

  # Add AMD-GPU
  # This will mostly work only on Linux, amd/rocm-smi is not yet supported on Windows
  try:
    out = subprocess.run(['amd-smi', 'list', '--csv'], shell=True, text=True, capture_output=True, check=False)
    if out.returncode == 0:
      install_requires.extend(extras_require["amd-gpu"])
  except:
    out = subprocess.run(['rocm-smi', 'list', '--csv'], shell=True, text=True, capture_output=True, check=False)
    if out.returncode == 0:
      install_requires.extend(extras_require["amd-gpu"])
  finally:
    pass


_add_gpu_requires()

setup(
  name="hanzo-net",
  version="0.1.20",
  description="Hanzo Network - Distributed AI compute network for running models locally and remotely",
  author="Hanzo AI",
  author_email="dev@hanzo.ai",
  url="https://github.com/hanzoai/net",
  packages=find_packages(where="src"),
  package_dir={"": "src"},
  install_requires=install_requires,
  extras_require=extras_require,
  package_data={"net": ["netchat/**/*"]},
  entry_points={"console_scripts": ["hanzo-net = net.main:run"]},
  classifiers=[
    "Development Status :: 3 - Alpha",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: MIT License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Operating System :: OS Independent",
  ],
  long_description="""
Hanzo Network - Distributed AI compute network for running models locally and remotely.

**Platform Support:**
- Linux: Full support
- macOS Intel (x86_64): Full support (CPU/GPU acceleration)
- macOS Apple Silicon (ARM64): Full support with MLX acceleration
- Windows: Experimental support
- Mobile: Web interface with WebGPU support (iOS/Android browsers)

**Mobile Access:**
Mobile devices can join the network via web interface with:
- WebGPU acceleration where supported
- QR code for easy network joining
- Responsive web UI optimized for mobile

**Installation:**
```bash
pip install hanzo-net
```

**Usage:**
```bash
hanzo net  # Starts node with QR code for mobile access
```
""",
  long_description_content_type="text/markdown",
  python_requires=">=3.10",
)
