# vibecoding-arena
Evaluating the Capabilities of LLM-Assisted Coding Tools in Developing Hyper-Structures on the Hyperware stack.

## Overview
This repository serves as a structured survey of various LLM-assisted coding tools, evaluating their effectiveness in achieving specific programming tasks. Each task is approached with different tools and prompting strategies to compare their capabilities and limitations.

## Repository Structure
```
.
├── tasks/                    # Each subdirectory is a specific task to accomplish
│   ├── task1/               # e.g., "Build a REST API"
│   │   ├── README.md        # Task description, acceptance criteria, evaluation metrics
│   │   ├── attempts/        # Different attempts with various tools
│   │   │   ├── claude/      # Claude-specific attempt
│   │   │   │   ├── prompt.md    # Prompt used
│   │   │   │   ├── session.md   # Session transcript
│   │   │   │   └── result/      # Resulting code/solution
│   │   │   ├── windsurf/    # Windsurf-specific attempt
│   │   │   └── kibitz/      # Kibitz-specific attempt
│   │   └── evaluation.md    # Comparative analysis of different attempts
│   └── task2/
├── tools/                    # Documentation about each tool being tested
│   ├── claude.md            # Tool-specific setup, capabilities, limitations
│   ├── windsurf.md
│   └── kibitz.md
└── evaluation/              # Overall analysis and findings
    ├── metrics.md           # Evaluation criteria and scoring system
    └── results.md           # Comparative analysis across all tasks
```

## Tools Being Evaluated

| Tool            | Website                          |  Type |
|-----------------|----------------------------------|-------|
| Claude Code     | [Link](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview) | CLI with MCP |
| Windsurf        | [Link](https://codeium.com/windsurf)  | IDE |
| Kibitz          | [Link](https://github.com/nick1udwig/kibitz)  | Chat with MCP tools |

## Evaluation Metrics
- Success Rate: Did the tool accomplish the task?
- Accuracy: How close was the solution to the requirements?
- Efficiency: Number of iterations/prompts needed
- Code Quality: Readability, maintainability, best practices
- Error Handling: How well does it handle edge cases?
- Documentation: Quality of generated documentation

## Contributing
To add a new task or tool evaluation:
1. Follow the directory structure outlined above
2. Include detailed prompts and session transcripts
3. Document any setup or environment requirements
4. Provide comprehensive evaluation based on the metrics

## License
See [LICENSE](LICENSE) for details.
