# Contributing to Beats TV

First off, thank you for considering contributing to Beats TV! It's people like you that make Beats TV such a great tool.

## How Can I Contribute?

### Reporting Bugs

- Check if the bug has already been reported.
- If not, open a new issue. Include a clear title, a detailed description, and as much relevant information as possible (OS, version, steps to reproduce).

### Suggesting Enhancements

- Open a new issue with the tag "enhancement".
- Describe the current behavior and what you would like to see instead.

### Pull Requests

1. Fork the repo and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. Ensure the test suite passes (`npm test`).
4. Make sure your code lints.
5. Issue that pull request!

## Development Setup

### Prerequisites

- Node.js (Latest LTS recommended)
- pnpm (`npm install -g pnpm`)
- Rust (for Tauri backend)

### Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/BeatsTV/open-tv.git
   cd open-tv
   ```
2. Install dependencies:
   ```bash
   pnpm install
   ```
3. Run in development mode:
   ```bash
   pnpm tauri dev
   ```

## Code of Conduct

Please be respectful and patient with other contributors. We follow the standard Contributor Covenant.
