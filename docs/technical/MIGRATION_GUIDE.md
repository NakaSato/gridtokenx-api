# Documentation Migration Guide

## Overview

This guide explains how to migrate from the old documentation structure to the new LLM-optimized structure.

## Old vs New Structure

### Old Structure (Deprecated)
```
docs/
├── 01-c4-model/              # Mixed diagrams
├── 02-data-flow-diagrams/    # Mixed diagrams
├── 03-architecture-guides/   # Mixed content
├── 04-planning-reference/    # Planning docs
├── 05-index-navigation/      # Navigation aids
├── 06-data-dictionary/       # Data definitions
├── anchor/                   # Anchor docs
└── architecture/             # Architecture
```

### New Structure (Current)
```
docs/
├── technical/
│   ├── architecture/         # All architecture docs
│   │   ├── system/          # System-level
│   │   ├── blockchain/      # Blockchain-specific
│   │   ├── frontend/        # Frontend-specific
│   │   └── backend/         # Backend-specific
│   ├── diagrams/            # All visual docs
│   │   ├── sequence/        # PlantUML sequences
│   │   ├── component/       # C4 components
│   │   ├── deployment/      # Infrastructure
│   │   └── flow/           # Data flows
│   ├── guides/              # How-to guides
│   │   ├── setup/          # Getting started
│   │   ├── development/    # Dev workflows
│   │   ├── operations/     # Ops procedures
│   │   └── testing/        # Test strategies
│   ├── reference/           # Quick references
│   │   ├── api/            # API docs
│   │   ├── data-models/    # Schemas
│   │   └── contracts/      # Smart contracts
│   └── specifications/      # Detailed specs
│       ├── requirements/    # What to build
│       ├── processes/       # How it works
│       └── interfaces/      # Integration points
└── [legacy files remain for reference]
```

## Migration Mapping

### From 01-c4-model/
```
OLD → NEW
01-c4-model/C4_LEVEL_1_*.puml → technical/diagrams/component/
01-c4-model/C4_LEVEL_2_*.puml → technical/diagrams/component/
01-c4-model/C4_LEVEL_3_*.puml → technical/diagrams/component/
01-c4-model/*.md → technical/diagrams/component/ (documentation)
```

### From 02-data-flow-diagrams/
```
OLD → NEW
STEP_*.puml → technical/diagrams/sequence/
DFD_*.puml → technical/diagrams/flow/
ARCHITECTURE_OVERVIEW_*.puml → technical/diagrams/sequence/
UC02_*.puml → technical/diagrams/sequence/
*.md (process docs) → technical/specifications/processes/
```

### From 03-architecture-guides/
```
OLD → NEW
ARCHITECTURE_GUIDE.md → technical/architecture/system/
BLOCKCHAIN_GUIDE.md → technical/architecture/blockchain/
POA_*.md → technical/architecture/blockchain/
SYSTEM_ARCHITECTURE.md → technical/architecture/system/
*_GUIDE.md → technical/guides/development/
```

### From anchor/
```
OLD → NEW
ANCHOR_PROGRAMS_*.md → technical/architecture/blockchain/
ANCHOR_ARCHITECTURE_*.md → technical/architecture/blockchain/
*.puml → technical/diagrams/component/
Contract interfaces → technical/reference/contracts/
```

### Root Level Files
```
OLD → NEW
DATA_DICTIONARY.md → technical/reference/data-models/
PROCESS_SPECIFICATIONS.md → technical/specifications/processes/
INITIALIZATION_SETUP.md → technical/guides/setup/
RUN_ANCHOR_TESTS.md → technical/guides/testing/
QUICK_TEST_GUIDE.md → technical/guides/testing/
LOCALHOST_WALLET_SETUP.md → technical/guides/setup/
```

## Migration Steps

### Phase 1: Copy and Organize (Current)
```bash
# 1. Create new structure
mkdir -p technical/{architecture,diagrams,guides,reference,specifications}

# 2. Copy C4 diagrams
cp 01-c4-model/C4_*.puml technical/diagrams/component/

# 3. Copy sequence diagrams
cp 02-data-flow-diagrams/STEP_*.puml technical/diagrams/sequence/
cp 02-data-flow-diagrams/ARCHITECTURE_*.puml technical/diagrams/sequence/

# 4. Copy flow diagrams
cp 02-data-flow-diagrams/DFD_*.puml technical/diagrams/flow/
```

### Phase 2: Consolidate Documentation
```bash
# Architecture docs
cp 03-architecture-guides/ARCHITECTURE_GUIDE.md technical/architecture/system/
cp 03-architecture-guides/BLOCKCHAIN_GUIDE.md technical/architecture/blockchain/

# Anchor docs
cp anchor/ANCHOR_PROGRAMS_*.md technical/architecture/blockchain/
cp anchor/ANCHOR_ARCHITECTURE_*.md technical/architecture/blockchain/

# Setup guides
cp INITIALIZATION_SETUP.md technical/guides/setup/
cp LOCALHOST_WALLET_SETUP.md technical/guides/setup/

# Testing guides
cp RUN_ANCHOR_TESTS.md technical/guides/testing/
cp QUICK_TEST_GUIDE.md technical/guides/testing/
```

### Phase 3: Create Index Files
```bash
# Create README for each section
touch technical/architecture/README.md
touch technical/diagrams/README.md
touch technical/guides/README.md
touch technical/reference/README.md
touch technical/specifications/README.md
```

### Phase 4: Add Metadata
Add frontmatter to all migrated files:
```yaml
---
title: Document Title
category: architecture|diagrams|guides|reference|specifications
migrated_from: old/path/to/file.md
last_updated: 2025-11-08
---
```

### Phase 5: Update Cross-References
```bash
# Find all links to old structure
grep -r "\[.*\](.*01-c4-model" technical/

# Update to new structure
# OLD: [Link](../01-c4-model/file.md)
# NEW: [Link](../diagrams/component/file.md)
```

### Phase 6: Validation
```bash
# Check all PlantUML files compile
find technical/diagrams -name "*.puml" -exec plantuml -checkonly {} \;

# Check for broken links
find technical -name "*.md" -exec grep -l "](.*" {} \;

# Verify directory structure
tree technical/
```

## Automated Migration Script

```bash
#!/bin/bash
# migrate-docs.sh

set -e

echo "Starting documentation migration..."

# Create new structure
mkdir -p technical/{architecture/{system,blockchain,frontend,backend},diagrams/{sequence,component,deployment,flow},guides/{setup,development,operations,testing},reference/{api,data-models,contracts},specifications/{requirements,processes,interfaces}}

# Migrate diagrams
echo "Migrating diagrams..."
cp 01-c4-model/C4_*.puml technical/diagrams/component/ 2>/dev/null || true
cp 02-data-flow-diagrams/STEP_*.puml technical/diagrams/sequence/ 2>/dev/null || true
cp 02-data-flow-diagrams/DFD_*.puml technical/diagrams/flow/ 2>/dev/null || true
cp 02-data-flow-diagrams/ARCHITECTURE_*.puml technical/diagrams/sequence/ 2>/dev/null || true

# Migrate architecture docs
echo "Migrating architecture docs..."
cp 03-architecture-guides/ARCHITECTURE_GUIDE.md technical/architecture/system/ 2>/dev/null || true
cp 03-architecture-guides/BLOCKCHAIN_GUIDE.md technical/architecture/blockchain/ 2>/dev/null || true
cp 03-architecture-guides/POA_*.md technical/architecture/blockchain/ 2>/dev/null || true

# Migrate Anchor docs
echo "Migrating Anchor documentation..."
cp anchor/ANCHOR_*.md technical/architecture/blockchain/ 2>/dev/null || true

# Migrate guides
echo "Migrating guides..."
cp INITIALIZATION_SETUP.md technical/guides/setup/ 2>/dev/null || true
cp LOCALHOST_WALLET_SETUP.md technical/guides/setup/ 2>/dev/null || true
cp RUN_ANCHOR_TESTS.md technical/guides/testing/ 2>/dev/null || true
cp QUICK_TEST_GUIDE.md technical/guides/testing/ 2>/dev/null || true

# Migrate reference
echo "Migrating reference docs..."
cp DATA_DICTIONARY.md technical/reference/data-models/ 2>/dev/null || true

# Migrate specifications
echo "Migrating specifications..."
cp PROCESS_SPECIFICATIONS.md technical/specifications/processes/ 2>/dev/null || true

echo "Migration complete!"
echo "Review technical/ directory and update cross-references"
```

## Post-Migration Tasks

### 1. Update Links
- [ ] Update all internal links
- [ ] Update README files
- [ ] Update navigation
- [ ] Test all links

### 2. Add Metadata
- [ ] Add frontmatter to all files
- [ ] Add last_updated dates
- [ ] Add category tags
- [ ] Add related_docs links

### 3. Create Missing Docs
- [ ] Create index files
- [ ] Create overview docs
- [ ] Create quick start guides
- [ ] Create migration notes

### 4. Validation
- [ ] All PlantUML files compile
- [ ] No broken links
- [ ] Consistent formatting
- [ ] Complete cross-references

### 5. Cleanup
- [ ] Archive old structure (optional)
- [ ] Update root README
- [ ] Update CONTRIBUTING guide
- [ ] Announce to team

## Benefits of New Structure

### For LLMs
✅ Clear hierarchical organization  
✅ Consistent naming conventions  
✅ Metadata for context understanding  
✅ Logical loading order  
✅ Cross-reference navigation  

### For Developers
✅ Intuitive navigation  
✅ Category-based organization  
✅ Quick reference access  
✅ Clear documentation types  
✅ Easy to maintain  

### For Project
✅ Better maintainability  
✅ Scalable structure  
✅ Version control friendly  
✅ Industry standard  
✅ Future-proof  

## Support

For questions or issues during migration:
1. Check this guide
2. Review technical/README.md
3. Check category README files
4. Ask the team

---
**Last Updated**: 2025-11-08  
**Migration Status**: Phase 1 Complete
