# ADR-0002: すべての自社コンポーネントをRustで実装する

- Status: Accepted
- Date: 2026-08-29

## 背景

プロジェクトはまだ prototype 段階で、凍結された public API はない。その目的は、確率的なモデル挙動の周囲に決定論的で検査可能な shell を置くことである。この早い段階で実装を TypeScript、Rust、IPC boundary に分割すると、それらの boundary が研究上の価値を生む前に、protocol と packaging の複雑性が増す。

CLI は、ローカル利用、CI、再現可能な評価、そして将来的な standalone distribution の主要インターフェースになることも期待されている。

## 決定

すべての first-party component を Rust で実装する。

- harness runtime と state machine
- Reasoning IR と epistemic type
- 決定論的 validator と verification pass
- evaluation と benchmark tooling
- CLI
- model-provider adapter
- 決定論的 oracle adapter
- 将来の desktop application
- 将来の optional MCP またはその他の integration adapter

外部モデルは trusted computing base の外側に残し、任意の provider がホストできる。モデルは Rust adapter boundary 経由で呼び出す。

通常の build、test、CLI、または将来の desktop execution に、Node.js、TypeScript、Python、その他の言語 runtime は不要とする。

## ワークスペースの構成

```text
crates/
  reasoning-harness-core/   trusted runtime primitives
  reasoning-harness-cli/    native `reason` executable
examples/                    language-neutral fixtures
```

追加の crate は、実際の ownership boundary が現れた場合に限って導入する。概念上の module を写すだけの大きな workspace は作らない。

## デスクトップへの影響

desktop client は延期するが、Rust-only の決定を維持しなければならない。Rust-native/Rust-first UI stack の候補は後で評価できる。この ADR によって toolkit を意図的に凍結することはない。

## 今この時点で決める理由

現在の migration cost は最小限である。prototype には小規模な typed IR、validator set、1つの framework trace、少数の test しかない。provider adapter、CLI contract、desktop work が存在してから言語を決定すると、大幅にコストが増える。

## 結果

### 利点

- runtime、CLI、eval、将来の UI で1つの type system を使える
- CLI と harness を接続するためだけの IPC boundary が不要
- 将来的に single-binary CLI を簡単に配布できる
- epistemic state と verdict に対して strong enum と exhaustive matching を使える
- deterministic tooling を小さく native に保てる
- cross-platform CI でユーザーが実行するものと同じ実装を試せる

### コスト

- 一部の model-provider SDK は Python や TypeScript の方が成熟している可能性がある
- provider integration には直接 HTTP 実装が必要になる場合がある
- scripting language よりも rapid experimentation に明示的な type が多く必要になることがある

これらのコストは許容する。provider SDK の利便性は研究上の問いの一部ではないためである。provider-specific code が correctness semantics を定義してはならない。
