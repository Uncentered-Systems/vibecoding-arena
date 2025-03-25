# Process Template Project Guidelines

## Build Commands
- **Hyperware Kit**: `kit bs` (shortcut for build and serve)
- **Hyperware Build**: `kit build`
- **Hyperware Serve**: `kit serve`
- **Rust Build**: `cargo build` / `cargo build --release`
- **UI Dev**: `cd ui && npm run dev` (port 3000)
- **Lint UI**: `cd ui && npm run lint`
- **Build & Package UI**: `cd ui && npm run build:copy`
- **Run Tests**: Configure in `test/tests.toml` and run with standard cargo test

## Code Style
### Rust
- **Naming**: snake_case for variables/functions, PascalCase for types/enums
- **Error Handling**: Use `anyhow::Result` and error! logging macros
- **Types**: Prefer strongly typed enums with descriptive variants
- **Documentation**: Document public APIs with /// comments

### TypeScript/React
- **TypeScript**: ES2020 target, use React 18 with function components
- **Imports**: Standard ES module imports
- **Formatting**: Use ESLint rules - eslint:recommended, typescript-eslint
- **Components**: Follow React hooks guidelines, avoid class components
- **State Management**: Use Zustand for state where appropriate

## Project Structure
- Separate Rust crates: process-template, shared-types
- UI folder contains React/TypeScript frontend
- WebAssembly build targeting Hyperware environment