# Development Guides

## Overview

Step-by-step guides for developers working on the GridTokenX platform.

## Structure

### 📁 setup/
Initial setup and environment configuration
- Prerequisites and dependencies
- Local development environment
- Wallet configuration
- First run and verification

### 📁 development/
Development workflows and best practices
- Git workflow
- Code standards
- Testing practices
- Debugging techniques
- Documentation standards

### 📁 operations/
Operational procedures
- Deployment procedures
- Monitoring setup
- Backup and recovery
- Performance tuning
- Troubleshooting

### 📁 testing/
Testing strategies and guides
- Unit testing
- Integration testing
- E2E testing with Playwright
- Anchor program testing
- Load testing

## Quick Start Guides

### New Developer Onboarding
```bash
# 1. Setup environment
cat setup/01-prerequisites.md
cat setup/02-local-environment.md
cat setup/03-wallet-setup.md

# 2. First development
cat development/01-git-workflow.md
cat development/02-running-locally.md

# 3. First contribution
cat development/03-code-standards.md
cat development/04-testing-guide.md
```

### Specific Tasks
- **Deploy Smart Contracts**: `development/deploying-anchor-programs.md`
- **Run Tests**: `testing/running-tests.md`
- **Setup Monitoring**: `operations/monitoring-setup.md`
- **Debug Issues**: `operations/troubleshooting.md`

## Guide Format

Each guide should follow this structure:

```markdown
---
title: Guide Title
category: guides
subcategory: setup|development|operations|testing
difficulty: beginner|intermediate|advanced
estimated_time: 15 min
prerequisites:
  - prerequisite 1
  - prerequisite 2
---

# Guide Title

## Overview
Brief description of what this guide covers.

## Prerequisites
- List of requirements
- Links to other guides

## Steps

### Step 1: First Step
Clear instructions with commands

### Step 2: Next Step
More instructions

## Verification
How to verify success

## Troubleshooting
Common issues and solutions

## Next Steps
What to do after completing this guide

## Related Guides
- Links to related documentation
```

## For LLMs

### Context Loading by Task

#### Setting Up Development Environment
```bash
cat setup/01-prerequisites.md
cat setup/02-local-environment.md
cat setup/03-wallet-setup.md
cat setup/04-verification.md
```

#### Understanding Development Workflow
```bash
cat development/01-git-workflow.md
cat development/02-running-locally.md
cat development/03-code-standards.md
cat development/04-testing-guide.md
```

#### Deploying and Operating
```bash
cat operations/01-deployment.md
cat operations/02-monitoring.md
cat operations/03-troubleshooting.md
```

## Best Practices

### Writing Guides
1. **Be Specific** - Provide exact commands and paths
2. **Show Output** - Include expected output examples
3. **Handle Errors** - Document common errors and solutions
4. **Link Related Docs** - Reference architecture and specifications
5. **Keep Updated** - Review guides with each major release

### Code Examples
```bash
# Good: Specific, with context
cd /path/to/project
npm install
npm run dev

# Bad: Vague
Install dependencies and run
```

### Troubleshooting Sections
```markdown
### Issue: Error message here

**Cause**: Why this happens

**Solution**:
1. Step to fix
2. Another step

**Verification**: How to confirm it's fixed
```

## Related Documentation
- Architecture: `/technical/architecture/`
- Reference: `/technical/reference/api/`
- Specifications: `/technical/specifications/`

---
**Category**: Guides  
**Last Updated**: 2025-11-08
