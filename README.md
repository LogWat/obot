# obot
ゲーム(OSU!(mania))用DiscordBot

## できること
- 新しくRanked, Loved, (Qualified)になったmapを自動的にメッセージとして表示
- 上記のmapを自動的にダウンロード

## TODO
- ダウンロード済みのmapをdbに登録(重複したダウンロード防止)
- 第3者がダウンロードしたmapを利用できるようにする
- cursorstring使ってDB全体更新用のコマンドを作成する(出力はなし)

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