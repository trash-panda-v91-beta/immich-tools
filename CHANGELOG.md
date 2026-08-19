# Changelog

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
