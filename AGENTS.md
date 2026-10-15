# Deno-to-Rust rewrite instructions

## Workspace model

This task runs in `/RustProjects/ogma3-tools`, which is a **host-created Git worktree** for the branch `rewrite/deno-to-rust`, based on `master` from the repository at `/RustProjects/ogma3-tools`.

- The reference implementation is at `/RustProjects/ogma3-tools`. Treat that path as read-only. Never edit, reformat, rename, delete, or generate files there.
- Make implementation changes only under the current workspace, `/WebstormProjects/Ogma3.Tools`.
- Do not run `git` commands or attempt to create/switch branches, stage changes, commit, push, merge, or open a PR. In Docker's host-worktree mode, Git metadata is not available to the agent. The human created the branch before the sandbox starts and will inspect and commit the changes from the host.
- Do not initialize a nested repository. `/WebstormProjects/Ogma3.Tools` is a working tree of the existing repository, not a new independent repository.
- The host may have mounted `/RustProjects/ogma3-tools` read-only at the same absolute path. Do not try to bypass that restriction.
- `AGENTS.md` and `opencode.jsonc` may be local task setup files. Do not modify them or include them in the implementation changes unless explicitly asked.

## OpenAPI schemas

Two OpenAPI schemas are available through environment variables:

- `OPENAPI_PUBLIC_URL`: the public API schema.
- `OPENAPI_INTERNAL_URL`: the private API schema.

Both URLs point to the host's local ASP.NET Core server. Access them through the provided environment variables rather than hardcoding URLs.

Before implementing API interactions:

1. Fetch both schemas and inspect their contents.
2. Identify which operations belong to the public API and which belong to the private API.
3. Preserve the distinction between public and private operations in the Rust implementation.
4. Use the schemas as the source of truth for paths, HTTP methods, request and response types, authentication requirements, and error responses.
5. Do not assume that an operation available in one schema is available in the other.
6. Do not log credentials, authentication tokens, or sensitive data from private API responses.

If either schema is unavailable, report the failure explicitly rather than silently proceeding as though both schemas were inspected.

## OpenAPI schema endpoint

The host serves the OpenAPI schema at the URL in `OPENAPI_[PUBLIC|INTERNAL]_URL`.

- From inside Docker Sandbox, use `host.docker.internal` for a host service. Do not use `localhost` or `127.0.0.1` in the URL inside the sandbox.
- Fetch the schema using an HTTP GET and inspect its response and OpenAPI version. For example:

  ```sh
  curl -fsS "$OPENAPI_PUBLIC_URL" -o /tmp/public-openapi.json
  ```

- Use the schema as the source of truth for API request/response shapes. Do not guess missing details.
- If `OPENAPI_URL` is unset, the endpoint is unreachable, or the response is not a valid schema, report the exact blocker and continue work that can be done safely.
- Schema access is read-only reference access, not permission to exercise live API operations. Do not call mutating endpoints, submit writes, or otherwise change the live service.
- Never place credentials or secrets in source files, logs, tests, or generated artifacts. Do not commit a downloaded live schema unless the project deliberately tracks it and that is necessary.

## Porting approach

Before implementing, inspect the existing source and project metadata in `/RustProjects/ogma3-tools` and the corresponding files in this workspace. Understand the tool's CLI contract, inputs, outputs, serialization, error handling, file paths, environment variables, API calls, timeouts, retries, sorting/order, and dependencies.

- Preserve observable behavior unless a change is necessary or explicitly justified. Record intentional behavior changes in the final summary.
- Build a parity checklist from the Deno implementation before coding.
- A perfect byte-for-byte output parity is not necessary. If a float value differs on the 5th decimal place, or a compression result is 20 bytes off, it's perfectly acceptable.
- Don't recreate Deno's security policies like `--allow-net` or `--allow-read`. Rust's security model is different, and that's OK.
- Prefer idiomatic stable Rust and a small, justified dependency set. Follow existing repository conventions where practical.
- Use types that reflect the OpenAPI schema. Keep request/response models and error handling explicit. Do not silently swallow errors or change output formats.
- Add Rust tests for important behavior and edge cases. Use fixtures or mocks for repeatable tests; ordinary unit tests should not depend on live service state.
- If the repository has no Rust project structure, establish an appropriate Cargo project, including `Cargo.toml` and `Cargo.lock` where appropriate, plus concise usage/build documentation.
- Do not add unrelated refactors, generated files, credentials, local caches, or machine-specific paths to the implementation.
- Do not delete the Deno implementation merely to make room for Rust unless replacing it in place is clearly required by the existing repository structure and the user's goal. Prefer an explicit, well-documented layout if both implementations can coexist.
- Every time you compress the session, update a `THOUGHTS.md` file in the repository root with the results of that compression (or create the file if it does not exist). That way, you will be
able to start from that checkpoint should you run out of tokens.

## Toolchain and validation

At the start, check whether `rustc`, `cargo`, `rust-analyzer`, and `deno` are available. If a required tool is missing, install it inside the sandbox only; do not install toolchains or packages on the host. Ask for or allow only the network destinations required for installation.

Run applicable checks and fix failures caused by the rewrite:

- `cargo fmt --all -- --check`
- `cargo check --locked` when a lockfile exists; otherwise `cargo check`
- `cargo test --locked` when a lockfile exists; otherwise `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings` if supported; otherwise run the applicable Clippy invocation and explain any unsupported option

Also run relevant existing project checks and compare observable behavior with the Deno implementation. If a check cannot run, state why. Do not claim a check passed unless it actually ran successfully.

## Final response

Report the implementation and compatibility decisions, key files changed, checks actually run and their results, remaining gaps, and any required host-side follow-up. Do not claim that changes have been committed or merged; the human reviews and commits the changes from the host after the sandbox session.
