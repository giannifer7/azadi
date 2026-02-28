---
name: Azadi Literate Programming
description: Guidelines and patterns for using the Azadi literate programming system and its macro translator.
---

# Azadi Literate Programming System

The **Azadi** system provides a powerful way to manage complex codebases through a **Literate Programming** approach. It allows you to define code structures as high-level macros that generate documentation-rich source files.

## Core Components

1.  **`azadi-macros`**: A macro translator and evaluator. It transforms custom macro definitions and calls into raw text (typically `noweb` chunks).
    -   Supported and extensible via Python-based macros (`--pydef`).
2.  **`azadi-noweb`**: A literate programming tool that extracts code chunks from literate source files (Markdown, Org-mode, etc.) into final source-code files.

## General Workflow Pattern

1.  **Macro Definition**: Define reusable macros that encapsulate common code patterns.
2.  **Macro Usage**: Call these macros within a literate document (e.g., Markdown).
3.  **Expansion (`azadi-macros`)**: Expand the macros into an intermediate document containing `noweb` chunk definitions.
4.  **Extraction (`azadi-noweb`)**: Extract the named chunks from the intermediate document into the target source files.

## Macro System Features

-   **Variable Interpolation**: Use `%(var)X` to insert variables into your macros.
-   **Nesting**: Macros can call other macros, allowing for deep abstraction.
-   **Python Integration**: Enable `--pydef` to use Python logic within your macro definitions for complex transformations.

## Literate Extraction (noweb)

Chunks are defined using:
```markdown
# <<chunk name>>=
code goes here
# @
```
*(Delimiters are configurable via `--open-delim`, `--close-delim`, and `--chunk-end`.)*

## Implementation Guidelines for Agents
-   **Separation of Concerns**: Treat the literate manifest as the primary source of truth for architectural structure.
-   **Documentation**: Use the Markdown structure of the manifest to explain the *why* behind the code chunks.
-   **Tooling Integration**: Azadi tools should be integrated into the project's build system (Ninja, Meson, Make) to ensure automated and consistent synchronization.

## Best Practices
-   **PATH Discovery**: Build systems should ideally look for `azadi-noweb` and `azadi-macros` in the system `PATH`.
-   **Configurable Tool Paths**: In development environments or specialized build wrappers, allow for a configurable location for the Azadi binaries, either through environment variables or build parameters.
-   **Relative Build Artifacts**: Direct macro expansions to a `build/` or temporary directory (using `--output`) to keep the source tree clean.
-   **Build Locks**: In highly parallel build environments (like Ninja), ensure the transformation pipeline is executed atomically to prevent race conditions.
