# tepra

![coverage](https://raw.githubusercontent.com/naa0yama/boilerplate-rust/badges/coverage.svg)
![test execution time](https://raw.githubusercontent.com/naa0yama/boilerplate-rust/badges/time.svg)

TEPRA ラベルプリンター向け REST API + WebUI サーバー

## 概要

[テプラ クリエイター](https://www.kingjim.co.jp/sp/tepra_creator/) Ver.4.20 が提供する
WebAPI 通信モジュール (`29108/tcp`) を経由してプリンター操作・印刷を行うサーバーです。

**主要機能:**

- `data/` に配置したテンプレートを WebUI から選択して印刷指示
- REST API 経由でテンプレートへのデータ流し込み印刷
- WebUI 上で OpenAPI ドキュメント参照
- 画像の WebUI / REST API 印刷

## クイックスタート

```bash
# HTTP サーバー起動 (引数なし — tepra.toml または built-in default を使用)
cargo run -p tepra-web -- serve

# --template-dir を明示 (default: templates/)
cargo run -p tepra-web -- serve --template-dir ./data

# bind / creator-base を明示
cargo run -p tepra-web -- serve \
  --template-dir ./data \
  --bind 127.0.0.1:3000 \
  --creator-base http://198.51.100.1:29108

# デフォルト値入りの tepra.toml を生成 (--path / --force で挙動制御)
cargo run -p tepra-web -- config init

# バージョン表示
cargo run -p tepra-web -- version
```

### tepra.toml (最小構成例)

プロジェクトルートに `tepra.toml` を配置すると `tepra serve` 単独で起動できる。
`tepra config init` でデフォルト値入りのテンプレートを生成できる (既存ファイルは
`--force` 未指定時は上書きしない)。

```toml
# tepra serve config file
# CLI arg > env var (TEPRA_*) > file > built-in default の順で上書き。

template_dir = "data"
bind = "0.0.0.0:3000"
creator_base = "http://localhost:29108"
```

詳細は [`docs/specs/components/tepra-web-cli.md`](docs/specs/components/tepra-web-cli.md) を参照。

## TEPRA WebAPI セットアップ

テプラ クリエイター Ver.4.20 に含まれる「WebAPI 用通信モジュール」を事前にセットアップしてください。
本サーバーはこのモジュールが提供する `29108/tcp` へ通信します。

デフォルトでは `127.0.0.1:29108` で待ち受けるため、Linux / WSL2 環境から利用する場合は
以下の手順でネットワーク全体に公開します。不要な場合はスキップしてください。

### サービス停止

```powershell
### 管理者 ###
PS> Stop-Service -Name "TpsWebPrintService"
```

### appsettings.json 書き換え (IPv4 All で待ち受け)

```powershell
### 管理者 ###

#Requires -Version 5.1
$ErrorActionPreference = 'Stop'

$Path   = 'C:\Windows\KING JIM\TEPRA WEBAPI\appsettings.json'
$NewUrl = 'http://0.0.0.0:29108'

if (-not (Test-Path -LiteralPath $Path)) { throw "見つかりません: $Path" }

$json = Get-Content -LiteralPath $Path -Raw -Encoding UTF8 | ConvertFrom-Json

$http = $json.Kestrel.Endpoints.Http
if (-not $http) { throw 'Kestrel.Endpoints.Http が存在しません。' }

if ($http.PSObject.Properties['Url']) {
    $old = $http.Url
    $http.Url = $NewUrl
} else {
    $old = '(なし)'
    $http | Add-Member -NotePropertyName Url -NotePropertyValue $NewUrl
}

$out = $json | ConvertTo-Json -Depth 64
[System.IO.File]::WriteAllText($Path, $out, [System.Text.UTF8Encoding]::new($false))

Write-Host "Url: $old -> $NewUrl"
```

### サービス再起動・疎通確認

```powershell
### 管理者 ###
PS> Restart-Service -Name "TpsWebPrintService"
PS> netstat -ano | Select-String -Pattern ":29108"

  TCP    0.0.0.0:29108    0.0.0.0:0    LISTENING    <pid>
```

### Windows Firewall 解放

```powershell
### 管理者 ###

New-NetFirewallRule `
  -DisplayName "Tepra Series Web Print Service (TEPWPAPI)" `
  -Direction Inbound `
  -Action Allow `
  -Protocol TCP `
  -LocalPort 29108 `
  -Program "C:\WINDOWS\KING JIM\TEPRA WEBAPI\TEPWPAPI.exe" `
  -RemoteAddress LocalSubnet `
  -Profile Domain,Private
```

`-RemoteAddress 192.0.2.0/24` のように接続元サブネットを制限することを推奨します。

## クレート構成

```
crates/
  tepra-core/   ドメイン型 + KING JIM WebAPI クライアント
  tepra/        REST API レイヤー (ルーター・ハンドラー・テンプレート)
  tepra-web/    HTTP サーバーエントリーポイント (バイナリ: tepra)
```

## ドキュメント

| ドキュメント                                                                                 | 内容                                            |
| -------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| [`docs/development.md`](docs/development.md)                                                 | 開発環境セットアップ・mise タスク・コミット規約 |
| [`docs/specs/`](docs/specs/)                                                                 | コンポーネント仕様・アーキテクチャ設計          |
| [`docs/adr/`](docs/adr/)                                                                     | アーキテクチャ決定記録 (ADR)                    |
| [`docs/specs/external/tepra-creator-webapi.md`](docs/specs/external/tepra-creator-webapi.md) | TEPRA Creator WebAPI 仕様                       |

## ライセンス

[LICENSE](./LICENSE) に記載のライセンスの下で公開。

### OpenObserve (サードパーティ)

Dev Container 起動時に [OpenObserve Enterprise Edition](https://openobserve.ai/) が
自動的にダウンロード・インストールされます。Enterprise 版は 200 GB/Day のインジェスト
クォータ内であれば無料で利用できます。ライセンスは
[EULA](https://openobserve.ai/enterprise-license/) を参照してください。
