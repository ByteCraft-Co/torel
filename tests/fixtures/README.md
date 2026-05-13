# Torel Fixtures

This directory stores stable `.torel` inputs for compiler tests. Fixtures should be small, named by behavior, and updated with the relevant parser/typechecker/codegen tests.

Use `valid/` for programs expected to pass and `invalid/` for programs expected to fail with a specific diagnostic.

Golden outputs for selected fixtures live in `tests/golden`.
