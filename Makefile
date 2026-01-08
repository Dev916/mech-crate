# MechCrate Project Makefile
# 🦝 Crate Raccoon

.PHONY: install uninstall test help

# Install mx command to /usr/local/bin
install:
	@./install.sh

# Uninstall mx command
uninstall:
	@echo "Removing mx from /usr/local/bin..."
	@rm -f /usr/local/bin/mx 2>/dev/null || sudo rm -f /usr/local/bin/mx
	@echo "✓ mx uninstalled"

# Test by creating a sample project
test:
	@echo "Testing mx new..."
	@rm -rf /tmp/mx-test-project
	@./bin/mx new /tmp/mx-test-project
	@echo ""
	@echo "Testing make doctor..."
	@cd /tmp/mx-test-project && make doctor
	@echo ""
	@echo "Testing mx add..."
	@cd /tmp/mx-test-project && $(CURDIR)/bin/mx add api
	@echo ""
	@echo "✓ All tests passed!"
	@rm -rf /tmp/mx-test-project

# Show help
help:
	@echo ""
	@echo "🦝 MechCrate Development"
	@echo ""
	@echo "Commands:"
	@echo "  make install    - Install mx to /usr/local/bin"
	@echo "  make uninstall  - Remove mx from /usr/local/bin"
	@echo "  make test       - Run basic tests"
	@echo "  make help       - Show this help"
	@echo ""
	@echo "Manual usage:"
	@echo "  ./bin/mx help   - Run mx directly"
	@echo ""
