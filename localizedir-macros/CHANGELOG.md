# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.2.0] - 2026-11-12

### Bug Fixes
- gate path-dependent test on Windows by @elcoosp ([826c4c4](https://github.com/elcoosp/localizedir/commit/826c4c424f99ce7b7c4bc455965fa59aeb9fbbd7))
- use forward slashes in temp_base for Windows by @elcoosp ([7fff3ce](https://github.com/elcoosp/localizedir/commit/7fff3ce6da944f7176f9d7f46fb2f8638b00d996))
- gate path-tokenstream test on windows by @elcoosp ([162fd76](https://github.com/elcoosp/localizedir/commit/162fd7691234cbc84ceb4e7eb0652088d566c569))
- remove file-level cfg gate, keep per-test gates by @elcoosp ([0f2a1ff](https://github.com/elcoosp/localizedir/commit/0f2a1ffb060a097be2c21bf74e8bd4f1e56205c2))
- gate entire macro test files for windows by @elcoosp ([8b47090](https://github.com/elcoosp/localizedir/commit/8b47090d8c4a7eb0d82015b9ce3a8686a1f8a70b))
- skip windows path tests by @elcoosp ([b4f3662](https://github.com/elcoosp/localizedir/commit/b4f3662cb3d1eca27b3e0c486af17fb18809c8ae))
- skip invalid utf-8 test on macOS (APFS forbids it) by @elcoosp ([de762ca](https://github.com/elcoosp/localizedir/commit/de762cada2e50fece5586d3ff7f9f48222ae7918))

### Documentation
- rewrite README for v0.2.0 API by @elcoosp ([efa330b](https://github.com/elcoosp/localizedir/commit/efa330bffea5ff6b8f12da2693190860c2a8362d))
- add readme by @elcoosp ([5bdd0f6](https://github.com/elcoosp/localizedir/commit/5bdd0f604c2149c989a9e277951d1ebe4c3adb35))

### Features
- macro crate by @elcoosp ([f7fb7a5](https://github.com/elcoosp/localizedir/commit/f7fb7a570d83e31e4d405c38d9a7ad11909421aa))
- impl, crash in doctest can not find dir by @elcoosp ([ee93426](https://github.com/elcoosp/localizedir/commit/ee934268a201510a0c3a9206692a5f3956da6b8c))

### Miscellaneous
- readme by @elcoosp ([484d8e4](https://github.com/elcoosp/localizedir/commit/484d8e4d6a8dc1349776052fac088b51d8cffdde))

### Refactor
- extract naming conventions into naming module by @elcoosp ([532982d](https://github.com/elcoosp/localizedir/commit/532982d4d72eda1ed4cd1a9f8ec0869d8dd1e4ef))
- reduce lib.rs to a thin proc-macro facade by @elcoosp ([587360e](https://github.com/elcoosp/localizedir/commit/587360e6d221fbb721ee04fb7261990c6768e35f))
- extract code generation into expand module by @elcoosp ([9dec469](https://github.com/elcoosp/localizedir/commit/9dec469578d58ba4d0d06de4713887b112aca87e))
- extract filesystem discovery into its own module by @elcoosp ([75eab91](https://github.com/elcoosp/localizedir/commit/75eab91dd5d6053f1b57d3a244fae20e30e3c446))
- extract attribute parsing into args module by @elcoosp ([6870c06](https://github.com/elcoosp/localizedir/commit/6870c067f5964cba81060a139b7a10639238380d))

### Testing
- snapshot: unknown option diagnostic by @elcoosp ([718ca53](https://github.com/elcoosp/localizedir/commit/718ca5353612e372edb1d82d0813f86e44717c92))
- add UI test: unknown attribute option is rejected by @elcoosp ([e2ac422](https://github.com/elcoosp/localizedir/commit/e2ac4229010d9186138300525e677618aa8227c4))
- snapshot: non-enum diagnostic by @elcoosp ([1e0e387](https://github.com/elcoosp/localizedir/commit/1e0e387be959d033f4f3563e7c646ecb5f6c0db4))
- add UI test: non-enum target is rejected by @elcoosp ([04320b0](https://github.com/elcoosp/localizedir/commit/04320b026ede31bc2096d3417f655e5d16f2231e))
- snapshot: non-literal attribute diagnostic by @elcoosp ([18a7245](https://github.com/elcoosp/localizedir/commit/18a7245dd789b16c122f2f60158eeecbf007c297))
- add UI test: non-literal attribute is rejected by @elcoosp ([cfb7482](https://github.com/elcoosp/localizedir/commit/cfb7482cf3a097c577a78d35017a9b29f2645510))
- snapshot: generic enum diagnostic by @elcoosp ([51c8f0d](https://github.com/elcoosp/localizedir/commit/51c8f0dcda890f401b608fca4c3ff657210a4bcb))
- add UI test: generic enum is rejected by @elcoosp ([1fb2f71](https://github.com/elcoosp/localizedir/commit/1fb2f71890c657a3fc723c8f6affe94c50effdb3))
- snapshot: fielded variant diagnostic by @elcoosp ([b148871](https://github.com/elcoosp/localizedir/commit/b1488711f4e143816f6e525097aa937ce8217cd6))
- add UI test: fielded variant is rejected by @elcoosp ([d266322](https://github.com/elcoosp/localizedir/commit/d266322cf6edc40fccd0ababcb17bf431304f59f))
- add trybuild UI test runner by @elcoosp ([6f01067](https://github.com/elcoosp/localizedir/commit/6f01067a7c105b97e86f657cea46e9025bd6239b))

### build
- update localizedir-macros package metadata by @elcoosp ([23974f7](https://github.com/elcoosp/localizedir/commit/23974f73ca217eaa4ebe6247f0a1bc9c7e758a76))
**Full Changelog**: https://github.com/elcoosp/localizedir/releases/tag/v0.2.0
