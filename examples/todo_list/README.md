# Todo List CLI

A command-line todo list application written in Crab.

## Overview

This example demonstrates:

- Classes and encapsulation (Task class with private fields)
- File I/O for persistent storage (JSON serialization)
- Command pattern for CLI operations
- Result/Option error handling
- Pattern matching with switch expressions
- Sealed classes for command types

## Building

```bash
cd examples/todo_list
crab build
```

## Usage

```bash
# Add a task
crab run add "Buy groceries" 5

# List all tasks
crab run list

# List only pending tasks
crab run list pending

# List by priority (highest first)
crab run list priority

# Mark task as completed
crab run done task_123456

# Reopen a completed task
crab run undo task_123456

# Remove a task permanently
crab run remove task_123456

# Clear all completed tasks
crab run clear

# Show statistics
crab run stats
```

## Architecture

```
src/
  main.crab          - CLI entry point and command parsing
```

### Key Components

- **Task**: Data model with id, title, completion status, priority
- **TaskRepository**: Handles persistence to JSON file
- **Command hierarchy**: Sealed class with concrete command implementations
- **Argument parser**: Pattern matching on command strings

## Data Storage

Tasks are stored in `~/.todo_list.json` as JSON array.
