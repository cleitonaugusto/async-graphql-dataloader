# Licensing

## Summary

| Version | License | Cost |
|---|---|---|
| 0.1.0 – 0.2.0 | MIT OR Apache-2.0 | free, permanently |
| 0.3.0 onward | AGPL-3.0-or-later **OR** [Commercial](COMMERCIAL-LICENSE.md) | free under AGPL / paid otherwise |

## The change at 0.3.0

Everything published up to and including 0.2.0 stays under MIT OR Apache-2.0.
Those grants are perpetual and irrevocable — they cannot be withdrawn, and this
project will not try to. If 0.2.0 does what you need, it remains free for any
use, forever, with no obligations beyond attribution.

From 0.3.0 the project moves to a dual license:

- **AGPL-3.0-or-later**, free for everyone. Use it, modify it, ship it — and if
  users interact with your version over a network, offer them your source under
  the same terms (AGPL-3.0 §13).
- **A commercial license**, paid, which lifts that source-disclosure
  obligation. See [COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md) for terms and
  pricing.

## Why

Maintaining this properly takes real time — 0.2.0 came out of finding and
fixing a dispatch deadlock that made `load` never return, plus building the
test suite that would have caught it. Dual licensing funds that work by asking
the organizations that build commercial products on top of it to contribute,
while leaving it free for everyone else.

The 0.3.0 boundary is deliberate. The license change applies to new work, not
to a bug fix held hostage: the fix for the 0.1.0 deadlock ships in 0.2.0 under
the permissive license, free, before any of this takes effect.

## Which do you need?

**The AGPL is enough** if you are evaluating or prototyping, doing personal or
academic work, licensing your own project under AGPL-3.0-or-later, or using the
library only in software you never convey and never expose over a network
beyond your organization.

**You need the commercial license** to ship this in a network-facing or
distributed product without releasing that product's source under the AGPL.

Not sure? Email **augusto.cleiton@gmail.com** and describe your setup. An
honest answer costs nothing, including when the honest answer is "the AGPL
covers you, you don't need to pay."

## Alternatives

If neither option fits, `async-graphql` ships its own DataLoader behind its
`dataloader` feature, under MIT OR Apache-2.0. For most `async-graphql`
projects it is the more sensible default anyway: maintained alongside the
framework, integrated with its context. This is said plainly rather than
buried — a license choice works better when the alternatives are on the table.

## Contributions

See [CONTRIBUTING.md](CONTRIBUTING.md). Contributions require a sign-off that
grants the maintainer the right to license the contribution commercially,
because dual licensing is not possible otherwise.

---

*Not legal advice. If you are deciding on behalf of a company, have counsel
read the actual license texts.*
