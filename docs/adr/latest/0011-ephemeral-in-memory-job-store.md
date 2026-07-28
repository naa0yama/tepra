# 0011. Ephemeral in-memory job store

- Status: Accepted
- Date: 2026-07-26
- Deciders: naa0yama

## Context

Jobs 機能では、印刷ジョブのリクエスト全体 (設定値 + テンプレ入力内容)・
印刷結果・プリンター・印刷時刻を保持し、`/ui/jobs` で一覧・詳細確認・
再印刷を提供したい。

Creator WebAPI には list-jobs エンドポイントが存在せず、`job_progress` の
完了後 retention も spec 未定義のため、印刷結果を後から API 経由で
遡って取得することはできない。したがって submit 時点でリクエスト内容と
outcome を tepra 側で記録する必要がある。

記録先として「永続化する (DB / ファイル)」か「プロセスメモリ上に保持する
(ephemeral)」の二択が生じる。プロセス再起動で履歴が消えることは、
その理由を文脈なしに見た開発者・運用者にとって驚きになりうるため、
明示的な決定として記録する。

## Decision

Jobs 機能のジョブ履歴を永続化せず、`AppState` 内の in-memory
`JobStore` (`Mutex<VecDeque<JobRecord>>`) に保持する。

- 上限 100 件、新しい順。超過分は古いものから破棄する。
- `JobRecord` は `record_id` (内部 monotonic ID、jobid ではない)・
  `printer`・`submitted_at` (epoch seconds)・`template`・
  `request: MergePrintRequest` (全体保存)・`outcome: JobOutcome`
  (`Accepted { jobid }` | `Failed { message }`) を保持する。
- 記録ポイントは `merge_print` orchestration 内の 1 箇所のみ
  (`client.print` の Ok/Err 双方)。UI `print_submit` と REST 経由 print の
  両方をこの 1 箇所で DRY にカバーする。
- 印刷結果は submit 時点の outcome をそのまま保存し、以降の背景ポーリング
  による状態更新は行わない。展開ビューで accepted job の live 進捗が
  必要な場合のみ既存 `job_card` route を lazy-load し、取得失敗時は
  保存済み outcome + 「進捗は期限切れ」表示に degrade する。
- プロセス再起動でジョブ履歴は消失する (ephemeral)。永続化は行わない。

## Consequences

- 容易になること: 実装が単純 (DB スキーマ・マイグレーション・永続化層が
  不要)。追加の I/O コストがなく `merge_print` のレイテンシに影響しない。
  メモリ使用量は上限 100 件で bound される。
- 難しくなること: プロセス再起動・クラッシュでジョブ履歴が失われる。
  複数プロセス/インスタンス間でジョブ履歴は共有されない
  (単一プロセス運用が前提)。長期の監査ログ用途には使えない。
- 運用上の含意: 履歴の永続性が必要になった場合 (監査要件・複数インスタンス
  化など) は本 ADR を Update し、DB や外部ストアへの移行を別決定として
  記録する。

## Alternatives Considered

- **DB/ファイルへの永続化**: プロセス再起動後も履歴が残る利点はあるが、
  スキーマ設計・マイグレーション・永続化層の実装コストが Jobs 機能の
  価値 (直近の印刷確認・再印刷) に対して過剰。却下。
- **各 accepted job を terminal まで背景ポーリングし結果を更新保存
  (アプローチ B)**: 履歴に最終的な終了状態が残る利点はあるが、
  poller のライフサイクル管理が複雑化する。却下 (`plans/2026-07-26-jobs-history-reprint.md`
  アプローチ比較 B)。
- **accepted のみ記録・失敗や全パラメータを非保持 (アプローチ C)**:
  実装は最小になるが、設定値・テンプレ入力保持という要件を満たさない。
  却下 (同上アプローチ比較 C)。

## History

- 2026-07-26: initial version
