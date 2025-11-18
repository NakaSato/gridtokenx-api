# GridTokenX API Gateway - Makefile
# Orchestrates testing, building, and deployment workflows

.PHONY: help all test test-unit test-integration test-e2e test-full coverage \
		run build clean docker-up docker-down migrate db-reset \
		lint format check install dev prod load-test

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
NC := \033[0m # No Color

# Configuration
API_BASE_URL ?= http://localhost:8080
CARGO := cargo
SCRIPTS_DIR := scripts

##@ Help

help: ## Display this help message
	@echo "$(BLUE)GridTokenX API Gateway - Available Commands$(NC)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make $(CYAN)<target>$(NC)\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  $(CYAN)%-25s$(NC) %s\n", $$1, $$2 } /^##@/ { printf "\n$(YELLOW)%s$(NC)\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Development

install: ## Install dependencies and build project
	@echo "$(BLUE)Installing dependencies...$(NC)"
	$(CARGO) build
	@echo "$(GREEN)✓ Dependencies installed$(NC)"

dev: ## Start development server
	@echo "$(BLUE)Starting development server...$(NC)"
	$(CARGO) run

build: ## Build release binary
	@echo "$(BLUE)Building release binary...$(NC)"
	$(CARGO) build --release
	@echo "$(GREEN)✓ Build complete: target/release/api-gateway$(NC)"

clean: ## Clean build artifacts
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	$(CARGO) clean
	@echo "$(GREEN)✓ Clean complete$(NC)"

format: ## Format code with rustfmt
	@echo "$(BLUE)Formatting code...$(NC)"
	$(CARGO) fmt
	@echo "$(GREEN)✓ Code formatted$(NC)"

lint: ## Run clippy linter
	@echo "$(BLUE)Running clippy...$(NC)"
	$(CARGO) clippy -- -D warnings
	@echo "$(GREEN)✓ Lint complete$(NC)"

check: ## Check code without building
	@echo "$(BLUE)Checking code...$(NC)"
	$(CARGO) check
	@echo "$(GREEN)✓ Check complete$(NC)"

##@ Database

migrate: ## Run database migrations
	@echo "$(BLUE)Running database migrations...$(NC)"
	$(CARGO) sqlx migrate run
	@echo "$(GREEN)✓ Migrations complete$(NC)"

db-reset: ## Drop and recreate database with migrations
	@echo "$(YELLOW)Resetting database...$(NC)"
	$(CARGO) sqlx database drop -y || true
	$(CARGO) sqlx database create
	$(CARGO) sqlx migrate run
	@echo "$(GREEN)✓ Database reset complete$(NC)"

##@ Testing - Sequential Test Suite

test-01-blockchain: ## 01. Test blockchain connection
	@echo "$(BLUE)========================================$(NC)"
	@echo "$(BLUE)Test 01: Blockchain Connection$(NC)"
	@echo "$(BLUE)========================================$(NC)"
	@bash $(SCRIPTS_DIR)/01-test-blockchain-connection.sh

test-02-flow: ## 02. Test complete user flow
	@echo "$(BLUE)========================================$(NC)"
	@echo "$(BLUE)Test 02: Complete Flow$(NC)"
	@echo "$(BLUE)========================================$(NC)"
	@bash $(SCRIPTS_DIR)/02-test-complete-flow.sh

test-03-meter: ## 03. Test meter verification flow
	@echo "$(BLUE)========================================$(NC)"
	@echo "$(BLUE)Test 03: Meter Verification$(NC)"
	@echo "$(BLUE)========================================$(NC)"
	@bash $(SCRIPTS_DIR)/03-test-meter-verification-flow.sh

test-04-minting: ## 04. Test token minting E2E
	@echo "$(BLUE)========================================$(NC)"
	@echo "$(BLUE)Test 04: Token Minting E2E$(NC)"
	@echo "$(BLUE)========================================$(NC)"
	@bash $(SCRIPTS_DIR)/04-test-token-minting-e2e.sh

test-05-clearing: ## 05. Test market clearing
	@echo "$(BLUE)========================================$(NC)"
	@echo "$(BLUE)Test 05: Market Clearing$(NC)"
	@echo "$(BLUE)========================================$(NC)"
	@bash $(SCRIPTS_DIR)/05-test-market-clearing.sh

test-06-settlement: ## 06. Test settlement flow
	@echo "$(BLUE)========================================$(NC)"
	@echo "$(BLUE)Test 06: Settlement Flow$(NC)"
	@echo "$(BLUE)========================================$(NC)"
	@bash $(SCRIPTS_DIR)/06-test-settlement-flow.sh

test-07-erc: ## 07. Test ERC lifecycle
	@echo "$(BLUE)========================================$(NC)"
	@echo "$(BLUE)Test 07: ERC Lifecycle$(NC)"
	@echo "$(BLUE)========================================$(NC)"
	@bash $(SCRIPTS_DIR)/07-test-erc-lifecycle.sh

test-08-integration: ## 08. Run integration tests
	@echo "$(BLUE)========================================$(NC)"
	@echo "$(BLUE)Test 08: Integration Tests$(NC)"
	@echo "$(BLUE)========================================$(NC)"
	@bash $(SCRIPTS_DIR)/08-run-integration-tests.sh

test-09-coverage: ## 09. Generate test coverage report
	@echo "$(BLUE)========================================$(NC)"
	@echo "$(BLUE)Test 09: Code Coverage$(NC)"
	@echo "$(BLUE)========================================$(NC)"
	@bash $(SCRIPTS_DIR)/09-test-coverage.sh

test-10-load: ## 10. Run load tests
	@echo "$(BLUE)========================================$(NC)"
	@echo "$(BLUE)Test 10: Load Testing$(NC)"
	@echo "$(BLUE)========================================$(NC)"
	@bash $(SCRIPTS_DIR)/10-load-test-api.sh

##@ Testing - Grouped Test Suites

test-unit: ## Run unit tests only
	@echo "$(BLUE)Running unit tests...$(NC)"
	$(CARGO) test --lib
	@echo "$(GREEN)✓ Unit tests complete$(NC)"

test-integration: test-01-blockchain test-02-flow test-03-meter ## Run integration tests (01-03)
	@echo "$(GREEN)✓ Integration tests complete$(NC)"

test-e2e: test-04-minting test-05-clearing test-06-settlement test-07-erc ## Run E2E tests (04-07)
	@echo "$(GREEN)✓ E2E tests complete$(NC)"

test-all-scripts: test-01-blockchain test-02-flow test-03-meter test-04-minting test-05-clearing test-06-settlement test-07-erc test-08-integration test-09-coverage ## Run all test scripts in sequence (01-09)
	@echo "$(GREEN)========================================$(NC)"
	@echo "$(GREEN)✓ All test scripts completed successfully!$(NC)"
	@echo "$(GREEN)========================================$(NC)"

test-full: test-unit test-all-scripts ## Run complete test suite (unit + all scripts)
	@echo "$(GREEN)========================================$(NC)"
	@echo "$(GREEN)✓ FULL TEST SUITE PASSED$(NC)"
	@echo "$(GREEN)========================================$(NC)"

test: test-unit ## Default: run unit tests (fast)
	@echo "$(GREEN)✓ Default tests complete$(NC)"
	@echo "$(YELLOW)Tip: Use 'make test-full' for comprehensive testing$(NC)"

coverage: test-09-coverage ## Alias for test-09-coverage

load-test: test-10-load ## Alias for test-10-load

##@ Quick Test Commands

test-quick: test-01-blockchain test-02-flow ## Quick smoke test (blockchain + flow)
	@echo "$(GREEN)✓ Quick tests complete$(NC)"

test-priority5: test-all-scripts ## Priority 5 testing suite (all scripts)
	@echo "$(GREEN)========================================$(NC)"
	@echo "$(GREEN)✓ PRIORITY 5 TEST SUITE COMPLETE$(NC)"
	@echo "$(GREEN)========================================$(NC)"

##@ Docker & Services

docker-up: ## Start all Docker services
	@echo "$(BLUE)Starting Docker services...$(NC)"
	docker-compose up -d
	@echo "$(GREEN)✓ Services started$(NC)"

docker-down: ## Stop all Docker services
	@echo "$(YELLOW)Stopping Docker services...$(NC)"
	docker-compose down
	@echo "$(GREEN)✓ Services stopped$(NC)"

docker-logs: ## View Docker service logs
	docker-compose logs -f

redis-setup: ## Setup Redis authentication
	@bash $(SCRIPTS_DIR)/setup-redis-auth.sh

##@ Production

prod: build ## Build and prepare for production
	@echo "$(BLUE)Preparing production build...$(NC)"
	@echo "$(GREEN)✓ Production build ready$(NC)"
	@echo "$(YELLOW)Run: target/release/api-gateway$(NC)"

deploy: ## Deploy to production (placeholder)
	@echo "$(RED)Production deployment not yet configured$(NC)"
	@echo "$(YELLOW)TODO: Configure production deployment$(NC)"

##@ Monitoring & Maintenance

health-check: ## Check API health
	@echo "$(BLUE)Checking API health...$(NC)"
	@curl -s $(API_BASE_URL)/health | jq . || echo "$(RED)✗ API not responding$(NC)"

logs: ## View application logs
	@echo "$(BLUE)Viewing application logs...$(NC)"
	tail -f logs/*.log 2>/dev/null || echo "$(YELLOW)No log files found$(NC)"

metrics: ## View Prometheus metrics
	@echo "$(BLUE)Fetching metrics...$(NC)"
	@curl -s $(API_BASE_URL)/metrics || echo "$(RED)✗ Metrics not available$(NC)"

##@ Special Targets

all: clean install migrate test-full build ## Complete workflow: clean, install, migrate, test, build
	@echo "$(GREEN)========================================$(NC)"
	@echo "$(GREEN)✓ Complete workflow finished!$(NC)"
	@echo "$(GREEN)========================================$(NC)"

ci: lint check test-unit ## CI pipeline tasks
	@echo "$(GREEN)✓ CI checks passed$(NC)"

pre-commit: format lint check ## Pre-commit checks
	@echo "$(GREEN)✓ Pre-commit checks passed$(NC)"

watch: ## Watch and rebuild on changes
	@echo "$(BLUE)Watching for changes...$(NC)"
	$(CARGO) watch -x run

##@ Information

version: ## Show version information
	@echo "$(BLUE)GridTokenX API Gateway$(NC)"
	@echo "Version: 0.1.1"
	@echo "Rust: $$(rustc --version)"
	@echo "Cargo: $$(cargo --version)"

status: health-check ## Check system status
	@echo ""
	@echo "$(BLUE)Database Status:$(NC)"
	@$(CARGO) sqlx migrate info 2>/dev/null || echo "$(YELLOW)Database not accessible$(NC)"
	@echo ""
	@echo "$(BLUE)Docker Status:$(NC)"
	@docker-compose ps 2>/dev/null || echo "$(YELLOW)Docker not running$(NC)"

env: ## Show environment configuration
	@echo "$(BLUE)Environment Configuration:$(NC)"
	@echo "API_BASE_URL: $(API_BASE_URL)"
	@echo "RUST_LOG: $${RUST_LOG:-info}"
	@echo "DATABASE_URL: $${DATABASE_URL:-not set}"
	@echo "REDIS_URL: $${REDIS_URL:-not set}"

##@ Documentation

docs: ## Generate and open documentation
	@echo "$(BLUE)Generating documentation...$(NC)"
	$(CARGO) doc --open --no-deps

api-docs: ## View OpenAPI documentation
	@echo "$(BLUE)Opening API documentation...$(NC)"
	@open $(API_BASE_URL)/swagger-ui/ 2>/dev/null || echo "$(YELLOW)Server may not be running$(NC)"

##@ Workflow Examples

example-dev: ## Example: Full development workflow
	@echo "$(BLUE)Example Development Workflow:$(NC)"
	@echo "  1. make install       # Install dependencies"
	@echo "  2. make db-reset      # Setup database"
	@echo "  3. make dev           # Start development server"
	@echo "  4. make test-quick    # Run quick tests"
	@echo "  5. make test-full     # Run full test suite"

example-ci: ## Example: CI/CD workflow
	@echo "$(BLUE)Example CI/CD Workflow:$(NC)"
	@echo "  1. make ci            # Run CI checks"
	@echo "  2. make test-full     # Full test suite"
	@echo "  3. make build         # Build release"
	@echo "  4. make deploy        # Deploy (TBD)"

example-testing: ## Example: Testing workflow
	@echo "$(BLUE)Example Testing Workflow:$(NC)"
	@echo "  1. make test-quick           # Quick smoke tests"
	@echo "  2. make test-integration     # Integration tests"
	@echo "  3. make test-e2e             # E2E tests"
	@echo "  4. make test-all-scripts     # All test scripts"
	@echo "  5. make coverage             # Generate coverage"
	@echo "  6. make load-test            # Load testing"
