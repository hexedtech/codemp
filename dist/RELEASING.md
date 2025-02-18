# Release flow

Releases on registries are automatically handled with GitHub Actions, which will only run on commits tagged as `v[0-9]+.[0-9]+.[0-9]+`.

Every time a new tag matching that is pushed:
- builds and tests core library
- publishes core rust library on `crates.io` (which will rebuild main documentation)
- builds codemp javascript bindings for windows, linux and macos and publishes it on `npm`
- builds codemp python bindings for windows, linux and macos and publishes it on `PyPI`
- builds codemp luajit bindings for windows, linux and macos and publishes it on both `LuaRocks` (source) and codemp.dev/releases (binary)
- builds codemp java bindings for windows, linux and macos and publishes it on `Maven Central`

# Checklist
Before writing the tag, make sure all these steps have been followed:

> do any required change in a branch named `release/vX.Y.Z`, for which a PR can then be opened

- [ ] `Cargo.toml`: bump version
- [ ] `dist/js/package.json`: make sure `version` matches with Cargo.toml (also check `optionalDependencies`: all versions must be the same)
- [ ] `dist/py/pyproject.toml`: make sure `version` matches with Cargo.toml
- [ ] `dist/java/build.gradle`: make sure `version` matches with Cargo.toml
- [ ] `dist/lua/codemp-X.Y.Z-1.rockspec`: make sure `version` matches with Cargo.toml
	- note that the rockspec file contains current version in its name so must be renamed
	- note that rockspec versions have a fourth component: "revision". we don't use it so always set is as `1`
- [ ] update `Cargo.lock` (basically delete it and re-run `cargo build`. check diff before committing!)
- [ ] **make sure the version you're about to release is available** (on all registries!)
- [ ] update last tag in "commits since last tag" badge (in README.md)
- [ ] commit all these changes (in `release/vX.Y.Z` branch), open a PR and have it approved and merged
- [ ] generate a new tag with same version as the one specified in Cargo.toml. include changelog in its description
	- use `git log <last-tag>..HEAD --oneline` to get a commit list, but **don't just put it as-is!**
	- **remember to push the newly generated tag with** (`git push --tags`)
