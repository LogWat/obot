# obot
ゲーム(OSU!(mania))用DiscordBot

## できること(やりたいこと)
- 新しくRanked, Loved, (Qualified)になったmapを自動的にメッセージとして表示(未実装)
- 登録されたユーザの情報追跡(未実装)

## フォルダ構成
```
obot/                           # cargoプロジェクトのルート
  ├── src/                      # ソースコード
  │    ├── main.rs              # main コマンド登録とか
  │    ├── cache.rs             # グローバルデータ
  │    ├── owner.rs             # 管理者判別用の関数とか
  |    ├── build.rs             # ビルド時に実行されるスクリプト
  │    ├── scheduler.rs         # スケジューラー
  |    ├── commands/            # コマンドの実装フォルダ
  |          ├── ...
  |    ├── web/                 # Webアクセス(api叩くとか)用のフォルダ
  |          ├── ...
  ├── migrations/               # dbのmigration

```

## メモ
- なぜ管理者専用のコマンドに`#[owners_only]`を使わないのか
    - 管理者じゃなかった場合のハンドルができないから