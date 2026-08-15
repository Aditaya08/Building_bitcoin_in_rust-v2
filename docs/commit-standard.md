# Commit Standard

This repository uses module-wise commits mapped to the guide chapters.

## Order

For each module:

1. Implementation commit
2. Test commit
3. Documentation commit
4. Follow-up fix commit, if verification finds a bug or formatting issue

## Message Format

```text
feat(scope): short implementation summary
test(scope): short test coverage summary
docs(scope): short documentation summary
fix(scope): short bug fix summary
```

Valid scopes:

```text
workspace
btclib
crypto
blockchain
network
miner
node
wallet
docs
ci
```

## Push Standard

Push after each meaningful commit or small group of related commits:

```bash
git push origin main
```

This keeps GitHub history reviewable and lets reviewers distinguish what was implemented, tested, documented, and fixed.
