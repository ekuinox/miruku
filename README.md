# miruku

a7iii から FTP で画像をアップロードする -> 画像をWebで確認できるように

## やりたいこと

- アップロードしたユーザに応じて付属情報みたいなのをつける
- サムネを生成する
- 画像に対してランダムなIDを振る
- 公開/非公開/リンク共有できるようにする
- cli からサムネを生成できるようにする
- FTP 用のディレクトリをウォッチする
- データ量をそこまで食わないようにする (symlink で済むならそうしたい)

## ディレクトリ構造

`media_id` ... メディアに対するUUID

```
./data
├── media
│   ├── {media_id}
│   │   ├── {origin_image}
│   │   ├── meta.toml
│   │   └── thumb.jpg
```

### `meta.toml` の中身

meta {
    id: MediaId
    visibility: (private, public)
    origin: string
    attributes: map<string, string>
}

以下のコマンドで `./source` から `./data` 下のファイルを生成する。

`$ miruku generate-media ./source`

## Server

以下のコマンドで `./data` を使ってサーバを `9999` ポートで開始する。

`$ miruku start-server`

### API

`GET /media/list` 
`GET /media/thumb/{media_id}`
`GET /media/origin/{media_id}`
`GET /media/meta/{media_id}`
