# Modular Architecture Rules

**CRITICAL**: All code MUST follow modular architecture principles.

## Module Structure
1. Types & Interfaces (First)
2. Constants (Second)
3. Pure Functions - Exported (Third)
4. Internal Helpers - Not exported (Fourth)

## Requirements
- Single Responsibility Principle
- Pure Functions First
- Dependency Injection
- Interface-Driven Design
- Test-Driven Development
- Explicit Exports

## Refactoring Triggers
- Function > 50 lines → Split
- File > 300 lines → Split into modules
- Duplicate code → Extract to utility
