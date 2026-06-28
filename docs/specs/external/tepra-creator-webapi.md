# TEPRA Creator WebAPI (External Dependency)

## Overview

KING JIM の `.NET` 製ローカル WebAPI モジュール。Windows ホスト上で
`http://localhost:29108` をリッスンし、USB/Bluetooth 接続された TEPRA
プリンタとの通信を担う。tepra-api はこの WebAPI を 1:1 facade する。

- **Vendor**: 株式会社キングジム
- **Endpoint base**: `http://localhost:29108/api/printer`
- **Transport**: HTTP/1.1 + JSON
- **Auth**: なし (localhost のみリッスン前提)
- **Platform**: Windows のみ (Creator アプリ同梱)

## Reference Sources

| Source   | Path                                               | Role                                                                                 |
| -------- | -------------------------------------------------- | ------------------------------------------------------------------------------------ |
| JS SDK   | `tmp/tc_webapi/tepraprint.js`                      | REST `/api/printer/*` 生 req/res の **一次ソース** (1598 行、全 13 endpoints 実装)   |
| PDF      | `tmp/tc_webapi/tepra_creator_webapi_reference.pdf` | Web 印刷 JavaScript API リファレンス Ver1.10 (2025-10-03)、enum 定数表 §3.1 / 利用例 |
| PDF text | `tmp/tc_webapi/tepra_creator_webapi_reference.txt` | 上記を `pdftotext -layout` で抽出                                                    |
| SDK zip  | `tmp/tc_webapi/tepra_creator_webapi_sdk.zip`       | 配布形態 (中身は `tepraprint.js` 1 ファイル)                                         |

PDF は **JS SDK 利用者向けドキュメント** であり、REST API 仕様書ではない。
REST 層の req/res 形は **必ず `tepraprint.js` ソースを正とする**。PDF は
enum 値定義 (errcode / TapeID / TapeCut / PrintSpeed / TapeKind /
StatusError / ImportFrameAttribute) と利用例の補足参照用。

## Endpoint Map

| #  | Method | Path                                      | Body | Note                                      |
| -- | ------ | ----------------------------------------- | ---- | ----------------------------------------- |
| 1  | GET    | `/api/printer`                            | —    | プリンタ一覧                              |
| 2  | GET    | `/api/printer/version`                    | —    | Creator WebAPI / プリンタドライバ version |
| 3  | GET    | `/api/printer/autoselect`                 | —    | 自動選択プリンタ                          |
| 4  | GET    | `/api/printer/info/{name}`                | —    | プリンタ詳細                              |
| 5  | GET    | `/api/printer/onlinestatus/{name}`        | —    | オンライン状態                            |
| 6  | GET    | `/api/printer/lwstatus/{name}`            | —    | LW ステータス (TTL キャッシュ対象)        |
| 7  | POST   | `/api/printer/print/{name}`               | JSON | 印刷ジョブ投入 (jobid 返却)               |
| 8  | GET    | `/api/printer/tapefeed/{name}?cutflag=`   | —    | テープ送り (cut あり/なし)                |
| 9  | GET    | `/api/printer/job/progress/{name}?jobid=` | —    | 進捗 polling (HTMX 1s)                    |
| 10 | GET    | `/api/printer/job/info/{name}?jobid=`     | —    | ジョブ詳細                                |
| 11 | POST   | `/api/printer/job/control/{name}`         | JSON | cancel 等                                 |
| 12 | POST   | `/api/printer/template/importframe`       | JSON | テンプレ枠取込                            |
| 13 | POST   | `/api/printer/getmargin/{name}`           | JSON | 余白計算                                  |

## Enum Constants (PDF §3.1)

JS SDK の `const TepraPrint*` で定義、数値 ID マッピング。Rust 側は
`#[serde(from/into = "u32")]` 等で透過変換。

| Enum                             | SDK 行 | PDF §  |
| -------------------------------- | ------ | ------ |
| `TepraPrintError`                | L17-   | §3.1.1 |
| `TepraPrintTapeID`               | L50-   | §3.1.2 |
| `TepraPrintTapeCut`              | L100-  | §3.1.3 |
| `TepraPrintPrintSpeed`           | L112-  | §3.1.4 |
| `TepraPrintTapeKind`             | L122-  | §3.1.5 |
| `TepraPrintStatusError`          | L174-  | §3.1.6 |
| `TepraPrintImportFrameAttribute` | L248-  | §3.1.7 |

## Tooling Notes

- PDF 抽出: `poppler-utils` (`pdftotext -layout`) 必須。初期は未インストール
  だったため `apt install poppler-utils` で導入済。再現性確保のため
  `.devcontainer` / `docker` に追加検討。
- PDF 著者: 岡山友輔氏 (Word 用 Acrobat PDFMaker 17 で生成)、改版履歴は
  Ver1.00 (2025-08-05) → Ver1.10 (2025-10-03)。

## Integration Strategy (tepra-api)

- 一次クライアント: `tepra-core::client::ReqwestTepraClient`
- テスト戦略: `wiremock` で Creator API モック起動 (T9a/T10a)、`MockTepraClient`
  trait で actor/handler ユニット (T8 以降)
- E2E は Windows 実機 + Creator + 実プリンタの手動スモークのみ、CI 除外
