#!/bin/bash

# Script to create a new task folder and copy markdown files

# Check if task name is provided
if [ $# -lt 1 ]; then
    echo "Usage: $0 <task-name> [attempt-name]"
    echo "Example: $0 new-task claude"
    exit 1
fi

# Set variables
TASK_NAME=$1
ATTEMPT_NAME=${2:-claude}  # Default to 'claude' if not provided
BASE_DIR="$(pwd)"
RESOURCES_DIR="$BASE_DIR/resources"
TASKS_DIR="$BASE_DIR/tasks"
NEW_TASK_DIR="$TASKS_DIR/$TASK_NAME"

# Define default attempts
DEFAULT_ATTEMPTS=("claude" "kibitz" "windsurf")

# Check if resources exist
if [ ! -f "$RESOURCES_DIR/api_documentation.md" ]; then
    echo "Error: api_documentation.md not found in $RESOURCES_DIR"
    exit 1
fi

if [ ! -f "$RESOURCES_DIR/HyperwareBook.md" ]; then
    echo "Error: HyperwareBook.md not found in $RESOURCES_DIR"
    exit 1
fi

# Create directory structure
echo "Creating task directory structure for '$TASK_NAME'..."
mkdir -p "$NEW_TASK_DIR"
mkdir -p "$NEW_TASK_DIR/attempts"

# If a specific attempt was provided, only create that one
if [ "$2" ]; then
    ATTEMPT_DIR="$NEW_TASK_DIR/attempts/$ATTEMPT_NAME"
    RESULT_DIR="$ATTEMPT_DIR/result"
    RESULT_RESOURCES_DIR="$RESULT_DIR/resources"
    
    mkdir -p "$ATTEMPT_DIR"
    mkdir -p "$RESULT_DIR"
    mkdir -p "$RESULT_RESOURCES_DIR"
    
    # Create prompt.md file
    cat > "$ATTEMPT_DIR/prompt.md" << EOL
# $TASK_NAME Task Prompt

## Objective
[Describe the objective of the task]

## Requirements
- [Requirement 1]
- [Requirement 2]
- [Requirement 3]

## Constraints
- [Constraint 1]
- [Constraint 2]

## Deliverables
- [Deliverable 1]
- [Deliverable 2]
EOL

    # Create empty session.md file
    touch "$ATTEMPT_DIR/session.md"
    
    # Copy markdown files
    echo "Copying markdown files to result/resources directory..."
    cp "$RESOURCES_DIR/api_documentation.md" "$RESULT_RESOURCES_DIR/"
    cp "$RESOURCES_DIR/HyperwareBook.md" "$RESULT_RESOURCES_DIR/"
else
    # Create all default attempts
    for attempt in "${DEFAULT_ATTEMPTS[@]}"; do
        ATTEMPT_DIR="$NEW_TASK_DIR/attempts/$attempt"
        RESULT_DIR="$ATTEMPT_DIR/result"
        RESULT_RESOURCES_DIR="$RESULT_DIR/resources"
        
        mkdir -p "$ATTEMPT_DIR"
        mkdir -p "$RESULT_DIR"
        mkdir -p "$RESULT_RESOURCES_DIR"
        
        # Create prompt.md file
        cat > "$ATTEMPT_DIR/prompt.md" << EOL
# $TASK_NAME Task Prompt

## Objective
[Describe the objective of the task]

## Requirements
- [Requirement 1]
- [Requirement 2]
- [Requirement 3]

## Constraints
- [Constraint 1]
- [Constraint 2]

## Deliverables
- [Deliverable 1]
- [Deliverable 2]
EOL

        # Create empty session.md file
        touch "$ATTEMPT_DIR/session.md"
        
        # Copy markdown files
        echo "Copying markdown files to result/resources directory for $attempt..."
        cp "$RESOURCES_DIR/api_documentation.md" "$RESULT_RESOURCES_DIR/"
        cp "$RESOURCES_DIR/HyperwareBook.md" "$RESULT_RESOURCES_DIR/"
    done
fi

# Create evaluation.md file
cat > "$NEW_TASK_DIR/evaluation.md" << EOL
# Evaluation Criteria for $TASK_NAME

## Functionality (40%)
- [ ] All required features are implemented
- [ ] No major bugs or issues

## Code Quality (30%)
- [ ] Code is well-organized and follows best practices
- [ ] Code is readable and maintainable
- [ ] Appropriate error handling

## Documentation (20%)
- [ ] Code is well-commented
- [ ] README provides clear instructions

## User Experience (10%)
- [ ] Interface is intuitive and user-friendly
- [ ] Application performs well
EOL

echo "Task directory structure created successfully!"
echo "New task location: $NEW_TASK_DIR"
echo ""
echo "Don't forget to update the prompt.md file with specific task details."

# List the created structure
echo ""
echo "Created directory structure:"
find "$NEW_TASK_DIR" -type d | sort
echo ""
echo "Created files:"
find "$NEW_TASK_DIR" -type f | sort