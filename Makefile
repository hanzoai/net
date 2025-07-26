# Makefile for hanzo-net
# Uses uv for Python management

# Variables
PYTHON_VERSION := 3.12
VENV := .venv
UV := uv
PYTHON := $(UV) run python
PIP := $(UV) pip
PROJECT_NAME := hanzo-net
SRC_DIR := src
TEST_DIR := test

# Colors for output
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m # No Color

# Default target: build and run tests
.PHONY: all
all: setup build test
	@echo "$(GREEN)✓ Build complete and tests passed$(NC)"

# Setup Python environment
.PHONY: setup
setup: install-uv install-python create-venv install-deps
	@echo "$(GREEN)✓ Setup complete$(NC)"

# Install uv if not present
.PHONY: install-uv
install-uv:
	@if ! command -v $(UV) &> /dev/null; then \
		echo "$(CYAN)Installing uv...$(NC)"; \
		curl -LsSf https://astral.sh/uv/install.sh | sh; \
	else \
		echo "$(GREEN)✓ uv is already installed$(NC)"; \
	fi

# Install Python using uv
.PHONY: install-python
install-python:
	@echo "$(CYAN)Installing Python $(PYTHON_VERSION)...$(NC)"
	@$(UV) python install $(PYTHON_VERSION)
	@echo "$(GREEN)✓ Python $(PYTHON_VERSION) installed$(NC)"

# Create virtual environment
.PHONY: create-venv
create-venv:
	@echo "$(CYAN)Creating virtual environment...$(NC)"
	@$(UV) venv $(VENV) --python $(PYTHON_VERSION)
	@echo "$(GREEN)✓ Virtual environment created$(NC)"

# Install dependencies
.PHONY: install-deps
install-deps:
	@echo "$(CYAN)Installing dependencies...$(NC)"
	@$(PIP) install -e .
	@echo "$(GREEN)✓ Dependencies installed$(NC)"

# Install development dependencies
.PHONY: install-dev
install-dev: install-deps
	@echo "$(CYAN)Installing development dependencies...$(NC)"
	@$(PIP) install pytest pytest-asyncio pytest-cov ruff mypy
	@echo "$(GREEN)✓ Development dependencies installed$(NC)"

# Build the project
.PHONY: build
build:
	@echo "$(CYAN)Building $(PROJECT_NAME)...$(NC)"
	@$(PYTHON) -m compileall $(SRC_DIR) -q
	@echo "$(GREEN)✓ Build successful$(NC)"

# Run the application
.PHONY: run
run:
	@echo "$(CYAN)Starting $(PROJECT_NAME)...$(NC)"
	@$(PYTHON) -m net.main

# Run with debug mode
.PHONY: run-debug
run-debug:
	@echo "$(CYAN)Starting $(PROJECT_NAME) in debug mode...$(NC)"
	@DEBUG=9 $(PYTHON) -m net.main

# Run tests
.PHONY: test
test:
	@echo "$(CYAN)Running tests...$(NC)"
	@if [ -d "$(TEST_DIR)" ] && [ -n "$$(find $(TEST_DIR) -name '*.py' -print -quit)" ]; then \
		$(PYTHON) -m pytest $(TEST_DIR) -v; \
	else \
		echo "$(YELLOW)⚠ No tests found in $(TEST_DIR)$(NC)"; \
	fi
	@# Also run tests in src directory
	@$(PYTHON) -m pytest $(SRC_DIR) -v --ignore=$(SRC_DIR)/net/tinychat || true

# Run tests with coverage
.PHONY: test-coverage
test-coverage:
	@echo "$(CYAN)Running tests with coverage...$(NC)"
	@$(PYTHON) -m pytest --cov=$(SRC_DIR)/net --cov-report=html --cov-report=term

# Lint code
.PHONY: lint
lint:
	@echo "$(CYAN)Linting code...$(NC)"
	@$(PYTHON) -m ruff check $(SRC_DIR)
	@echo "$(GREEN)✓ Linting complete$(NC)"

# Format code
.PHONY: format
format:
	@echo "$(CYAN)Formatting code...$(NC)"
	@$(PYTHON) -m ruff format $(SRC_DIR)
	@echo "$(GREEN)✓ Formatting complete$(NC)"

# Type check
.PHONY: type-check
type-check:
	@echo "$(CYAN)Running type checks...$(NC)"
	@$(PYTHON) -m mypy $(SRC_DIR) --ignore-missing-imports || true
	@echo "$(GREEN)✓ Type checking complete$(NC)"

# Compile gRPC files
.PHONY: grpc
grpc:
	@echo "$(CYAN)Compiling gRPC files...$(NC)"
	@cd $(SRC_DIR)/net/networking/grpc && \
		$(PYTHON) -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. node_service.proto && \
		sed -i '' "s/import node_service_pb2/from . &/" node_service_pb2_grpc.py
	@echo "$(GREEN)✓ gRPC compilation complete$(NC)"

# Clean build artifacts
.PHONY: clean
clean:
	@echo "$(CYAN)Cleaning build artifacts...$(NC)"
	@find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	@find . -type d -name "*.egg-info" -exec rm -rf {} + 2>/dev/null || true
	@find . -type f -name "*.pyc" -delete 2>/dev/null || true
	@find . -type f -name "*.pyo" -delete 2>/dev/null || true
	@find . -type d -name ".pytest_cache" -exec rm -rf {} + 2>/dev/null || true
	@find . -type d -name ".mypy_cache" -exec rm -rf {} + 2>/dev/null || true
	@find . -type d -name "htmlcov" -exec rm -rf {} + 2>/dev/null || true
	@find . -type f -name ".coverage" -delete 2>/dev/null || true
	@echo "$(GREEN)✓ Clean complete$(NC)"

# Deep clean including virtual environment
.PHONY: clean-all
clean-all: clean
	@echo "$(CYAN)Removing virtual environment...$(NC)"
	@rm -rf $(VENV)
	@echo "$(GREEN)✓ Deep clean complete$(NC)"

# Show Python version
.PHONY: python-version
python-version:
	@$(PYTHON) --version

# Install specific model support
.PHONY: install-apple
install-apple:
	@echo "$(CYAN)Installing Apple Silicon support...$(NC)"
	@$(PIP) install mlx mlx-lm
	@echo "$(GREEN)✓ Apple Silicon support installed$(NC)"

# Configure MLX for Apple Silicon
.PHONY: configure-mlx
configure-mlx:
	@echo "$(CYAN)Configuring MLX...$(NC)"
	@if [ -f "configure_mlx.sh" ]; then \
		./configure_mlx.sh; \
	else \
		echo "$(RED)✗ configure_mlx.sh not found$(NC)"; \
	fi

# Run formatting check (for CI)
.PHONY: format-check
format-check:
	@echo "$(CYAN)Checking code formatting...$(NC)"
	@$(PYTHON) -m ruff format --check $(SRC_DIR)

# Development server with auto-reload
.PHONY: dev
dev:
	@echo "$(CYAN)Starting development server...$(NC)"
	@$(UV) run watchmedo auto-restart \
		--directory=$(SRC_DIR) \
		--pattern="*.py" \
		--recursive \
		-- python -m net.main

# Show help
.PHONY: help
help:
	@echo "$(CYAN)$(PROJECT_NAME) Makefile$(NC)"
	@echo ""
	@echo "$(YELLOW)Usage:$(NC)"
	@echo "  make              - Setup, build and test (default)"
	@echo "  make setup        - Setup Python environment and install dependencies"
	@echo "  make build        - Build the project"
	@echo "  make run          - Run the application"
	@echo "  make test         - Run tests"
	@echo ""
	@echo "$(YELLOW)Development:$(NC)"
	@echo "  make dev          - Run with auto-reload (requires watchdog)"
	@echo "  make run-debug    - Run with debug logging"
	@echo "  make lint         - Run linter"
	@echo "  make format       - Format code"
	@echo "  make type-check   - Run type checker"
	@echo "  make test-coverage - Run tests with coverage report"
	@echo ""
	@echo "$(YELLOW)Installation:$(NC)"
	@echo "  make install-dev  - Install development dependencies"
	@echo "  make install-apple - Install Apple Silicon support"
	@echo "  make configure-mlx - Configure MLX for Apple Silicon"
	@echo ""
	@echo "$(YELLOW)Utilities:$(NC)"
	@echo "  make grpc         - Compile gRPC files"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make clean-all    - Clean everything including venv"
	@echo "  make python-version - Show Python version"
	@echo ""

# Quick start for new developers
.PHONY: quickstart
quickstart: setup install-dev
	@echo ""
	@echo "$(GREEN)✓ Quickstart complete!$(NC)"
	@echo ""
	@echo "$(CYAN)To run $(PROJECT_NAME):$(NC)"
	@echo "  make run"
	@echo ""
	@echo "$(CYAN)For development:$(NC)"
	@echo "  make dev    # Auto-reload on changes"
	@echo "  make test   # Run tests"
	@echo "  make lint   # Check code style"
	@echo ""

# CI/CD target
.PHONY: ci
ci: setup build lint format-check test
	@echo "$(GREEN)✓ CI checks passed$(NC)"

.DEFAULT_GOAL := all