# Development Guide

## Dev Container 環境 (VS Code)

Docker と VS Code Dev Containers 拡張機能が必要。

1. リポジトリをクローン:

```bash
git clone <repository-url>
cd tepra
```

2. VS Code でプロジェクトを開く:

```bash
code .
```

3. コマンドパレット (`Ctrl+Shift+P` / `Cmd+Shift+P`) から
   「Dev Containers: Reopen in Container」を選択

## devcontainer CLI + traefik 環境 (WSL2)

VS Code を使わず tmux + claude code などで複数 worktree / 複数プロジェクトを
並行開発する場合は devcontainer CLI と traefik を組み合わせます。
ポート衝突なしに各 devcontainer へ `p<port>.<branch>.<project>.localhost:8080` で
アクセスできます。

### 前提条件

WSL2 ホストに以下が必要。ユーザーグローバルの mise で管理します
(プロジェクトの `mise.toml` には含まれていません)。

```bash
mise use --global node@lts
mise use --global devcontainer-cli@0.86.0
```

`dev:up` 実行時にホストで `ssh-agent` と `gpg-agent` が起動していれば
SSH エージェント転送と GPG 署名がコンテナ内で自動的に有効になります。

### 初回セットアップ (WSL2 ホストで 1 回だけ)

```bash
mise run traefik:setup
```

traefik の状態確認:

```bash
systemctl --user status traefik
```

### devcontainer 起動・停止

```bash
mise run dev:up      # 現在の worktree の devcontainer を起動
mise run dev:down    # 停止・削除
mise run dev:exec    # 稼働中の devcontainer に bash で接続
mise run dev:status  # 稼働中の devcontainer 一覧
```

`dev:up` は `devcontainer.json` の `portsAttributes` に定義された全ポートに対して
traefik ルーティングを自動設定します。起動後に以下の URL が表示されます:

```
http://p<port>.<branch>.<project>.localhost:8080
```

例 (ポート `3000`、ブランチ `feature/add-ui`、プロジェクト `tepra`):

```
http://p3000.feature-add-ui.tepra.localhost:8080
```

### 複数 worktree での利用

```bash
cd /path/to/tepra
mise run dev:up
# -> http://p3000.main.tepra.localhost:8080

cd /path/to/tepra-feat
mise run dev:up
# -> http://p3000.feature-x.tepra.localhost:8080
```

ブランチ名は DNS ラベル形式 (小文字英数字・ハイフン・63 文字以内) に自動変換されます。

## mise タスク一覧

| タスク                | コマンド                    | 説明                                |
| --------------------- | --------------------------- | ----------------------------------- |
| Setup                 | `mise run setup`            | 初期セットアップ                    |
| Build                 | `mise run build`            | デバッグビルド                      |
| Build (release)       | `mise run build:release`    | リリースビルド                      |
| Build (timings)       | `mise run build:timings`    | ビルド時間計測                      |
| Check                 | `mise run check`            | cargo check                         |
| Test                  | `mise run test`             | テスト実行                          |
| TDD watch             | `mise run test:watch`       | ウォッチモード                      |
| Doc tests             | `mise run test:doc`         | ドキュメントテスト                  |
| Trace test            | `mise run test:trace`       | トレーステスト                      |
| Format                | `mise run fmt`              | フォーマット                        |
| Format check          | `mise run fmt:check`        | フォーマットチェック                |
| Lint (clippy)         | `mise run clippy`           | Lint                                |
| Lint strict           | `mise run clippy:strict`    | Lint (warnings をエラー扱い)        |
| Lint                  | `mise run lint`             | 総合 Lint                           |
| Lint (GitHub Actions) | `mise run lint:gh`          | actionlint                          |
| AST rules             | `mise run ast-grep`         | ast-grep カスタムルール             |
| Pre-commit (required) | `mise run pre-commit`       | コミット前チェック                  |
| Pre-push              | `mise run pre-push`         | プッシュ前チェック                  |
| Coverage              | `mise run coverage`         | カバレッジ計測                      |
| Coverage (HTML)       | `mise run coverage:html`    | HTML レポート                       |
| Audit                 | `mise run audit`            | セキュリティ監査                    |
| Deny (licenses/deps)  | `mise run deny`             | ライセンス・依存関係チェック        |
| Miri (UB detection)   | `mise run miri`             | 未定義動作検出                      |
| Clean (full)          | `mise run clean`            | 全クリーン                          |
| Clean (sweep)         | `mise run clean:sweep`      | 不要ファイル削除                    |
| Playwright setup      | `mise run setup:playwright` | Playwright MCP ブラウザインストール |

### コミット前チェック

```bash
mise run pre-commit
# clean:sweep + fmt:check + clippy:strict + ast-grep + lint:gh + check:no-plans
```

## コミット規約

Conventional Commits: `<type>(<scope>): <description>`

| type       | 用途                       |
| ---------- | -------------------------- |
| `feat`     | 新機能                     |
| `update`   | 既存機能の改善             |
| `fix`      | バグ修正                   |
| `chore`    | 動作変更なしのメンテナンス |
| `docs`     | ドキュメント               |
| `test`     | テスト                     |
| `refactor` | リファクタリング           |
| `ci`       | CI/CD                      |

## VSCode 拡張機能

Dev Container に含まれる拡張機能:

### Rust 開発

- **[rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)** — Rust 言語サポート
- **[CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)** — デバッガー
- **[Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml)** — Cargo.toml サポート
- **[Coverage Gutters](https://marketplace.visualstudio.com/items?itemName=ryanluker.vscode-coverage-gutters)** — カバレッジ表示

### コード品質

- **[dprint](https://marketplace.visualstudio.com/items?itemName=dprint.dprint)** — フォーマッター
- **[EditorConfig](https://marketplace.visualstudio.com/items?itemName=EditorConfig.EditorConfig)** — エディター設定統一
- **[Error Lens](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens)** — エラーインライン表示

### その他

- **[Mermaid Chart](https://marketplace.visualstudio.com/items?itemName=MermaidChart.vscode-mermaid-chart)** — Mermaid ダイアグラム
- **[YAML](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-yaml)** — YAML サポート
- **[Container Tools](https://marketplace.visualstudio.com/items?itemName=ms-azuretools.vscode-containers)** — Docker 操作

## Troubleshooting

### Rust デバッグ

```bash
RUST_LOG=trace RUST_BACKTRACE=1 cargo run -p tepra-web -- serve
```
