# agents-pool (private)

> This directory is a git submodule pointing to
> `git@github.com:claude-genasis/agents-pool.git` (private).
> General users will not see this content after `git clone`.

## Purpose

Curation pipeline for the genasis agents catalog:
1. **Crawl** — fetch latest agent .md files from community repos (ECC, wshobson, VoltAgent, dl-ezo, 0xfurai)
2. **Verify** — validate frontmatter, check for conflicts with genasis overlay injection
3. **Publish** — copy verified files to `../agents/base/` in the genasis public repo

## Usage (developer only)

```bash
cd agents-pool/
./scripts/crawl.sh      # shallow-clone all source repos
./scripts/verify.sh     # validate + copy to verified/
./scripts/publish.sh    # copy to ../agents/base/

# Then in genasis root:
cd ..
git add agents/
git commit -m "feat(agents): update catalog from pool"
git tag agents-v1.1.0
git push --tags         # CI creates release + tarball
```

## Configuration

Edit `config.toml` to add/remove source repositories.
