# Changelog

## [0.3.1](https://github.com/trash-panda-v91-beta/immich-tools/compare/v0.3.0...v0.3.1) (2026-08-20)


### Features

* **pcloud:** tag uploaded assets with folder ([#54](https://github.com/trash-panda-v91-beta/immich-tools/issues/54)) ([0ed8a55](https://github.com/trash-panda-v91-beta/immich-tools/commit/0ed8a5527b7416c23870b3d2bbb8236904b91db5))


### Bug Fixes

* **pcloud:** per-file timeout, retries and progress logging ([#52](https://github.com/trash-panda-v91-beta/immich-tools/issues/52)) ([00998bd](https://github.com/trash-panda-v91-beta/immich-tools/commit/00998bd048973fa99d4cb6a5c9e0f824cda4099d))

## [0.3.0](https://github.com/trash-panda-v91-beta/immich-tools/compare/v0.2.4...v0.3.0) (2026-08-20)


### ⚠ BREAKING CHANGES

* **deps:** Update toml ( 0.9.12+spec-1.1.0 ➔ 1.1.4 ) ([#49](https://github.com/trash-panda-v91-beta/immich-tools/issues/49))
* **deps:** Update cargo ([#35](https://github.com/trash-panda-v91-beta/immich-tools/issues/35))

### Features

* **deps:** Update cargo ([#35](https://github.com/trash-panda-v91-beta/immich-tools/issues/35)) ([2b888d3](https://github.com/trash-panda-v91-beta/immich-tools/commit/2b888d335c431c267e840da1e55689a506701567))
* **deps:** update mise tools ([#20](https://github.com/trash-panda-v91-beta/immich-tools/issues/20)) ([2320aff](https://github.com/trash-panda-v91-beta/immich-tools/commit/2320aff09a8d4e4692823479d6289db8a7e85bb4))
* **deps:** Update toml ( 0.9.12+spec-1.1.0 ➔ 1.1.4 ) ([#49](https://github.com/trash-panda-v91-beta/immich-tools/issues/49)) ([aa39cf5](https://github.com/trash-panda-v91-beta/immich-tools/commit/aa39cf5d520327c7922d91bd32bddca61c5d8abc))
* **pcloud:** drop config file, sync folder per request ([#48](https://github.com/trash-panda-v91-beta/immich-tools/issues/48)) ([73575ed](https://github.com/trash-panda-v91-beta/immich-tools/commit/73575ed4d6c97ddd06f011b16d056f2719b8f545))


### Bug Fixes

* **pcloud:** normalize folder paths to NFD ([#47](https://github.com/trash-panda-v91-beta/immich-tools/issues/47)) ([86974ee](https://github.com/trash-panda-v91-beta/immich-tools/commit/86974ee33fde2c786f6349dd72177b49a9cccec2))

## [0.2.4](https://github.com/trash-panda-v91-beta/immich-tools/compare/v0.2.3...v0.2.4) (2026-08-19)


### Bug Fixes

* **pcloud:** parse u64 file ids and hashes ([#44](https://github.com/trash-panda-v91-beta/immich-tools/issues/44)) ([7c8d443](https://github.com/trash-panda-v91-beta/immich-tools/commit/7c8d44341c5b6f79dddeeb7e27849cef0d5cab1a))

## [0.2.3](https://github.com/trash-panda-v91-beta/immich-tools/compare/v0.2.2...v0.2.3) (2026-08-19)


### Bug Fixes

* bundle CA certs into the scratch image ([#42](https://github.com/trash-panda-v91-beta/immich-tools/issues/42)) ([594c840](https://github.com/trash-panda-v91-beta/immich-tools/commit/594c840c1c67402d321d0059b7c73778d40342b0))

## [0.2.2](https://github.com/trash-panda-v91-beta/immich-tools/compare/v0.2.1...v0.2.2) (2026-08-19)


### Features

* **serve:** require bearer token on the HTTP API ([#40](https://github.com/trash-panda-v91-beta/immich-tools/issues/40)) ([c2e147b](https://github.com/trash-panda-v91-beta/immich-tools/commit/c2e147b099ea4df2d77e50302e604237b67acca2))

## [0.2.1](https://github.com/trash-panda-v91-beta/immich-tools/compare/v0.2.0...v0.2.1) (2026-08-19)


### Features

* **container:** update rust ( 1.87 ➔ 1.97 ) ([#37](https://github.com/trash-panda-v91-beta/immich-tools/issues/37)) ([3b6a074](https://github.com/trash-panda-v91-beta/immich-tools/commit/3b6a07459758e5aceefd5fe6fac4de64f207486d))
* **serve:** run favorites sync in the background ([#39](https://github.com/trash-panda-v91-beta/immich-tools/issues/39)) ([3c6541e](https://github.com/trash-panda-v91-beta/immich-tools/commit/3c6541ea5749f88343db1a58021c46290733e580))


### Continuous Integration

* **github-action:** pin jdx/mise-action action to 3c2e0cf ([#32](https://github.com/trash-panda-v91-beta/immich-tools/issues/32)) ([e373a3c](https://github.com/trash-panda-v91-beta/immich-tools/commit/e373a3c0ff00436ab34a4a3e25553cecc4162c6a))

## [0.2.0](https://github.com/trash-panda-v91-beta/immich-tools/compare/v0.1.1...v0.2.0) (2026-08-19)


### ⚠ BREAKING CHANGES

* **github-action:** Update github-actions ([#36](https://github.com/trash-panda-v91-beta/immich-tools/issues/36))

### Features

* **deps:** update cargo ([#34](https://github.com/trash-panda-v91-beta/immich-tools/issues/34)) ([5a943c4](https://github.com/trash-panda-v91-beta/immich-tools/commit/5a943c4a9605a2ef92639c82842b80689581e554))
* **sync-favorites:** run on a configurable interval ([#31](https://github.com/trash-panda-v91-beta/immich-tools/issues/31)) ([fb54b63](https://github.com/trash-panda-v91-beta/immich-tools/commit/fb54b63dbd09e4d8fb23a3b41e3df007381bd895))


### Continuous Integration

* **github-action:** Update github-actions ([#36](https://github.com/trash-panda-v91-beta/immich-tools/issues/36)) ([9a19c38](https://github.com/trash-panda-v91-beta/immich-tools/commit/9a19c38a8196595d4d3770db1cfcbbccaee06fc1))

## [0.1.1](https://github.com/trash-panda-v91-beta/immich-tools/compare/v0.1.0...v0.1.1) (2026-08-19)


### Features

* add HTTP API to manage pcloud folders and trigger sync ([#29](https://github.com/trash-panda-v91-beta/immich-tools/issues/29)) ([4035a6e](https://github.com/trash-panda-v91-beta/immich-tools/commit/4035a6ecd169b6ca7f435f9adebbaeb941a217d6))
* add sync-pcloud command to upload pCloud folders to Immich ([c4b9f33](https://github.com/trash-panda-v91-beta/immich-tools/commit/c4b9f33b0eb8c4669c89e8396e283f80f7f80cae))
* immich-tools with sync-favorites and watch-upload commands ([388ad13](https://github.com/trash-panda-v91-beta/immich-tools/commit/388ad137353217e47d3139be93231fe7950bd401))
* stream, hash-dedup, and parallelize pcloud sync ([#11](https://github.com/trash-panda-v91-beta/immich-tools/issues/11)) ([640dc92](https://github.com/trash-panda-v91-beta/immich-tools/commit/640dc92850fae752bf37be08f68669a01ab6447f))


### Bug Fixes

* add connect and request timeouts to HTTP client ([#8](https://github.com/trash-panda-v91-beta/immich-tools/issues/8)) ([b476636](https://github.com/trash-panda-v91-beta/immich-tools/commit/b4766365011d6204cf9b65de5c26e6cc8d94174b))
* **ci:** use version tags for actions instead of stale SHAs ([ab675eb](https://github.com/trash-panda-v91-beta/immich-tools/commit/ab675eb960da8596e25ace9afad2e4293196fcab))
* **docker:** add openssl-libs-static for musl static linking ([8b50cf8](https://github.com/trash-panda-v91-beta/immich-tools/commit/8b50cf81e7ab04bffc46712b5a1a4b13777c2bf1))
* stream asset downloads to disk to avoid OOM on large files ([7f6c917](https://github.com/trash-panda-v91-beta/immich-tools/commit/7f6c91766f7d948714ad5dca1330ae85ae89c78d))
* **sync-pcloud:** auth via access_token param, configurable EU API host ([96ac274](https://github.com/trash-panda-v91-beta/immich-tools/commit/96ac2743c60edc3870d449d67f3eb398ada28584))


### Continuous Integration

* add release-please releases and unify workflow naming ([b3a06c7](https://github.com/trash-panda-v91-beta/immich-tools/commit/b3a06c7b88d911587dce79122aee7c9da751b4f8))
* add release-please releases and unify workflow naming ([86c375c](https://github.com/trash-panda-v91-beta/immich-tools/commit/86c375cdd172bc2f642402ae5531b680ebf8326b))
* carry over release-type and docker-build fixes ([#3](https://github.com/trash-panda-v91-beta/immich-tools/issues/3)) ([4ea3e6a](https://github.com/trash-panda-v91-beta/immich-tools/commit/4ea3e6a3b9181532ca763b16c471f8da2dbdb497))
* rename workflows to code-quality-checks/pr-quality-checks, add renovate ([#7](https://github.com/trash-panda-v91-beta/immich-tools/issues/7)) ([20c8e2d](https://github.com/trash-panda-v91-beta/immich-tools/commit/20c8e2d5a6bf7d31f13e274ea215013f2dcc0fb8))
* support approve/cl4 and approve/trashbot-9000 labels in auto-approve ([#4](https://github.com/trash-panda-v91-beta/immich-tools/issues/4)) ([2b1e68c](https://github.com/trash-panda-v91-beta/immich-tools/commit/2b1e68c9be40dfe68034d804ced2a476df684246))
* switch to native amd64/arm64 runners, drop QEMU ([04959c4](https://github.com/trash-panda-v91-beta/immich-tools/commit/04959c49feea4b46f6fc15c8bb1932fd46e88562))
* use cl4 bot token for release-please ([df60b00](https://github.com/trash-panda-v91-beta/immich-tools/commit/df60b007da4ff1326759ef4de47c44134c4738ea))
