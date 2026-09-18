# Contributing

Bug reports, reproductions and pull requests are welcome. Please open an issue
before starting significant work, so we don't both build the same thing.

## Before you send code: the licensing requirement

From version 0.3.0 this project is dual-licensed — AGPL-3.0-or-later for
everyone, plus a paid commercial license that lifts the AGPL's
source-disclosure obligation. See [LICENSING.md](LICENSING.md).

That model only works if a single party can license the whole codebase under
both sets of terms. A contribution offered under the AGPL alone could never be
included in a commercially licensed build, which would fracture the project
into a part that can be sold and a part that cannot.

So, by opening a pull request, you agree to the following.

### Contributor terms

1. **You wrote it, or you have the right to send it.** The contribution is
   your original work, or you have obtained the rights necessary to submit it
   under these terms. If your employer has rights to work you produce, you have
   their permission.

2. **You grant a copyright license.** You grant Cleiton Augusto Correa Bezerra
   a perpetual, worldwide, non-exclusive, royalty-free, irrevocable copyright
   license to reproduce, modify, distribute and sublicense your contribution,
   **including the right to license it under terms of the maintainer's choosing,
   commercial terms among them**.

3. **You grant a patent license.** You grant the same parties a perpetual,
   worldwide, non-exclusive, royalty-free, irrevocable patent license covering
   any patent claims you own that your contribution necessarily infringes.

4. **You keep your copyright.** This is a license, not an assignment. You
   remain the author and owner of your contribution and may use it elsewhere
   however you like.

5. **No warranty.** You provide the contribution "as is", without warranty of
   any kind.

Sign off each commit to record your agreement:

```bash
git commit -s -m "your message"
```

This appends a `Signed-off-by:` line, the same convention the Linux kernel
uses. A pull request without sign-off cannot be merged.

If any of this does not work for you, please open an issue describing the
change instead of sending a patch. A well-written bug report is genuinely
valuable, and it carries no licensing question at all.

## Development

```bash
cargo test                  # unit, integration and doc tests
cargo clippy -- -D warnings # CI gate: must be clean
cargo fmt -- --check        # CI gate: must be clean
cargo run --example basic_usage
```

CI runs all four on every push. Please make sure they pass locally first.

### Tests around batching

`tests/batching.rs` bounds every `load` in a timeout on purpose. Version 0.1.0
shipped a dispatch bug that made `load` never resolve; a plain `.await` in a
test would have hung the suite instead of failing it. Keep new tests that await
a `load` inside a timeout so a regression fails loudly.
