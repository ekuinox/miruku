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

mediaId: string

media/{mediaId}/meta.toml

meta {
    id: MediaId
    visibility: (private, public)
    origin: string
    attributes: map<string, string>
}

media/{mediaId}/thumb.jpg
media/{mediaId}/{meta.origin}

`$ miruku import-media ./source ./data/`

## Server

`$ miruku start ./data -p 9999`

`GET /media/list` 
`GET /media/thumb/{imageId}`
`GET /media/origin/{imageId}`
`GET /media/meta/{imageId}`
