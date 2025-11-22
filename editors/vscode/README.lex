We will begin the planning of the vscode plugin for Lex here.

See the development milestones at the end of this document.

For context : 

	- Read the editors README [../README.lex] for the general view.
    - Initial release contemplates the core features as described in the crate's lib[lex-lsp/src/lib.rs]

Principles: 

	- No significant logic in the vscode plugin code.
    - Leverage all the LSP and only LSP for the first release.
	- For now can forgo the binary dependency on lex-lsp and use a hardcoded path for the binary.

  Tech Stack

  Core Build System:
  - TypeScript 5.7+ with ES2022 target
  - Node 20+ (matching rust-analyzer's approach)
  - npm for package management
  - ESBuild for bundling (fast, minimal config, used by rust-analyzer)

  Module System:
  - Node16 module resolution (required by vscode-languageclient 9.x)
  - ES modules with "type": "module" in package.json

  Runtime Dependencies:
  - vscode-languageclient v9.x (official LSP client library)

  Development Dependencies:
  - @vscode/test-electron (official headless testing tool)
  - @types/vscode (API types)
  - @types/node
  - esbuild (bundler)
  - eslint + @typescript-eslint/* (linting)
  - prettier (formatting, optional but rust-analyzer uses it)
  - TypeScript 5.7+

  Testing Framework:
  - Plain assertions or lightweight assertion library for unit tests
  - @vscode/test-electron for integration tests
  - BATS for shell-level orchestration (matching your Neovim pattern)

  File Layout

  editors/vscode/
  ├── src/
  │   ├── main.ts              # Extension entry point (activates LSP client)
  │   ├── client.ts            # LanguageClient setup and lifecycle
  │   └── config.ts            # Extension configuration handling
  ├── test/
  │   ├── unit/
  │   │   ├── index.ts         # Test loader (finds *.test.ts files)
  │   │   └── config.test.ts   # Unit tests for config module
  │   ├── integration/
  │   │   ├── lsp_handshake.test.ts     # Verify LSP initialization
  │   │   ├── lsp_hover.test.ts         # Test hover requests/responses
  │   │   ├── lsp_document_symbols.test.ts
  │   │   ├── lsp_folding_ranges.test.ts
  │   │   └── lsp_formatting.test.ts
  │   ├── fixtures/
  │   │   └── example.lex      # Test documents
  │   ├── runTests.ts          # Test runner using @vscode/test-electron
  │   └── lex_vscode_extension.bats    # BATS wrapper for CI
  ├── out/                     # Compiled JS output (gitignored)
  ├── dist/                    # Bundled extension (gitignored)
  ├── package.json
  ├── tsconfig.json
  ├── eslint.config.mjs
  ├── .vscodeignore           # Files to exclude from VSIX
  └── README.md

  Testing Strategy

  Unit Tests:
  - Located in test/unit/*.test.ts
  - Test configuration parsing, utility functions
  - No VS Code API required, fast execution
  - Custom test loader in test/unit/index.ts that glob-matches *.test.ts

  Integration Tests:
  - Located in test/integration/*.test.ts
  - Verify LSP handshake succeeds
  - Send LSP requests and validate responses (structure, not semantics)
  - Check that document changes trigger appropriate server notifications
  - Run in headless VS Code instance via @vscode/test-electron

  BATS Orchestration:
  - Shell out to npm test from BATS tests
  - Similar pattern to Neovim: each test runs headless with fixtures
  - Provides consistent CI interface across both editor plugins
  - Formats: junit for CI, pretty for local development

  Test Runner Implementation:
  - test/runTests.ts uses @vscode/test-electron
  - Downloads/caches VS Code binary
  - Launches headless with --disable-extensions
  - Points to extension development path and test suite entry point

  What Gets Tested:
  - Extension activation doesn't throw
  - LSP client connects to lex-lsp server
  - Document changes are synchronized
  - Capabilities are correctly advertised
  - LSP requests return well-formed responses
  - We do NOT test semantic correctness (that's lex-lsp's job)

  Development Workflow

  Local Setup:
  cd editors/vscode
  npm install
  npm run build        # Compile TypeScript
  npm run watch        # Continuous compilation

  Live Testing (F5 debugging):
  - Press F5 in VS Code to launch Extension Development Host
  - Opens new window with extension loaded
  - Set breakpoints in TypeScript source
  - Uses tsconfig.json sourcemaps for debugging

  Headless Testing:
  npm test                           # Run all tests
  npm run test:unit                  # Unit tests only
  npm run test:integration           # Integration tests only
  ./test/run_suite.sh --format=simple   # BATS wrapper, friendly output
  ./test/run_suite.sh --format=junit    # CI output

  Packaging:
  npm run package      # Creates .vsix file via vsce

  Scripts in package.json:
  {
    "scripts": {
      "build": "tsc -p .",
      "watch": "tsc -watch -p .",
      "bundle": "esbuild src/main.ts --bundle --outfile=dist/extension.js --external:vscode --format=cjs 
  --platform=node --target=node20",
      "vscode:prepublish": "npm run bundle -- --minify",
      "pretest": "npm run build",
      "test": "node ./out/test/runTests.js",
      "lint": "eslint src test",
      "format": "prettier --write 'src/**/*.ts' 'test/**/*.ts'"
    }
  }

  CI Integration:
  - GitHub Actions workflow installs Node, runs npm ci, executes npm test
  - BATS output in junit format for test result parsing
  - Xvfb wrapper on Linux (handled by @vscode/test-electron)

  Hardcoded LSP Binary Path:
  - Configuration option in package.json contributions
  - Default points to ../../target/debug/lex-lsp (relative to extension root)
  - Users can override via settings for custom builds

  Key Decisions Rationale

  ESBuild over Webpack: Rust-analyzer switched to ESBuild for speed and simplicity. Single-file bundling
  works for our thin wrapper.

  Custom test loader over Mocha: Rust-analyzer demonstrates this pattern. Gives us control without
  framework overhead.

  BATS wrapper: Matches your Neovim infrastructure. Single command interface for CI.

  No @vscode/test-cli: While newer, rust-analyzer uses @vscode/test-electron directly for more control
  (running against multiple VS Code versions). We can keep it simpler initially.

  Minimal dependencies: Only vscode-languageclient in production. Everything else is LSP protocol and
  standard library.

  Sources:
  - https://code.visualstudio.com/api/working-with-extensions/testing-extension
  - https://code.visualstudio.com/api/language-extensions/language-server-extension-guide
  - https://github.com/rust-lang/rust-analyzer/blob/master/editors/code/package.json
  - https://www.npmjs.com/package/@vscode/test-electron
  - https://github.com/microsoft/vscode-test


Development Milestones

	For each milestone, we will have a commit and push, together with working tests when relevant. Use this milestone name in the commit message.

	1. Node Environment Setup:

		- The package.json, tsconfig.json, eslint.config.mjs and other required confs.
		- The init of npm and install
		- The build and watch script
        - A stub unit test that exercises the infrastructure.

	2. GH Workflow
		- We will create the github workflow for the plugin.
		- This is to be tested and verified with real workflow runs.
		- You can merge to main in order to have the workflow picked up, and once it's working do work on a new branch for the rest of the work.

	3. Integration Tests Setup: 
		- A basic test case that exercises that integration is working (no Lex work, just the vscode level integration)
		- This means that it must work on the CI as well.
	4. Lex Basic Integration: 
		- Test the LSP handshake with the lex-lsp server.
	5. Features
		With the full setup working, verified both locally and on the CI we can start the actual feature work.
		For each feature 1-7 in lex-lsp/src/lib.rs: 
		- Implement the feature in the plugin
		- Write integration tests
        - Commit. 

		Critical: how can we be sure this is really working? How do we test with certainly that syntax hilighting is working? Or the Outline view is working? Or the folding ranges.
		This is a quite niche problem, if it's tricker than you'd expect, search online , see how projects like rust-analyzer do it.
		Bellow a few potential tips on testing follows.



 Retry Strategy:

  async function waitForSymbols(uri: vscode.Uri, maxAttempts = 10, delayMs = 500): 
  Promise<vscode.DocumentSymbol[]> {
    for (let i = 0; i < maxAttempts; i++) {
      const result = await vscode.commands.executeCommand<vscode.DocumentSymbol[]>(
        'vscode.executeDocumentSymbolProvider',
        uri
      );
      if (result && result.length > 0) {
        return result;
      }
      await new Promise(resolve => setTimeout(resolve, delayMs));
    }
    throw new Error('Document symbols not available after timeout');
  }

  Proposed Test Structure

  test/integration/
  ├── lsp_semantic_tokens.test.ts
  │   - Test semantic tokens legend is advertised
  │   - Test semantic tokens response structure
  │   - Test token count is non-zero for fixture
  │
  ├── lsp_document_symbols.test.ts
  │   - Test document symbols response structure
  │   - Test hierarchical nesting (children arrays)
  │   - Test known symbols exist in fixture
  │   - Test symbol kinds match expected values
  │
  └── fixtures/
      └── lsp-fixture.lex (symlink to specs/v1/benchmark/050-lsp-fixture.lex)

  Example Test Pattern

  // test/integration/lsp_semantic_tokens.test.ts

  import * as vscode from 'vscode';
  import * as assert from 'assert';
  import * as path from 'path';

  suite('LSP Semantic Tokens', () => {
    let document: vscode.TextDocument;

    suiteSetup(async () => {
      const fixtureUri = vscode.Uri.file(
        path.join(__dirname, '../fixtures/lsp-fixture.lex')
      );
      document = await vscode.workspace.openTextDocument(fixtureUri);
      await vscode.window.showTextDocument(document);

      // Wait for LSP to fully initialize
      await new Promise(resolve => setTimeout(resolve, 2000));
    });

    test('Returns semantic tokens legend', async () => {
      const legend = await vscode.commands.executeCommand<vscode.SemanticTokensLegend>(
        'vscode.provideDocumentSemanticTokensLegend',
        document.uri
      );

      assert.ok(legend, 'Legend should exist');
      assert.ok(legend.tokenTypes.length > 0, 'Should have token types');
      assert.ok(legend.tokenModifiers.length >= 0, 'Should have token modifiers');
    });

    test('Returns semantic tokens for document', async () => {
      const tokens = await vscode.commands.executeCommand<vscode.SemanticTokens>(
        'vscode.provideDocumentSemanticTokens',
        document.uri
      );

      assert.ok(tokens, 'Tokens should exist');
      assert.ok(tokens.data, 'Tokens should have data array');
      assert.ok(tokens.data.length > 0, 'Should have non-zero tokens');

      // Validate structure (data is integer array of deltas)
      assert.ok(tokens.data.every(n => Number.isInteger(n)), 'All deltas should be integers');

      console.log(`Received ${tokens.data.length} token deltas`);
    });
  });

