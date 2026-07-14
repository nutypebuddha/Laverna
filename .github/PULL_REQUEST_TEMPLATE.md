## Description

<!-- Briefly describe what this PR does. -->

## Type

<!-- Check one: -->

- [ ] Bug fix
- [ ] New feature
- [ ] Refactor
- [ ] Documentation
- [ ] CI/CD
- [ ] Dependency update

## Checklist

- [ ] `cargo fmt -- --check` passes
- [ ] `cargo clippy -- -D warnings` passes (default features)
- [ ] `cargo clippy --features llm -- -D warnings` passes
- [ ] `cargo deny check` passes
- [ ] `cargo test` passes
- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat(scope):`, `fix(scope):`)
- [ ] No new warnings introduced
- [ ] Pure functions only (no side effects in core logic)

## Related Issues

<!-- Link related issues: Fixes #123, Closes #456 -->
