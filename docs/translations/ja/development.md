---
source: d0d174df9061792086d11223ac4029e9d23f9d56
---

# 開発 { #development }

このページは、エディターをソースからビルドしたい方、アドオンを変更したい方、ドキュメントを書きたい方向けです。

## ビルド { #building }

サブモジュール付きでクローンします。`godot/addons/func_godot`のGodotアドオンがサブモジュールです。

```sh
git clone --recursive https://github.com/Paraxdev/TrenchDot.git
```

付けずにクローンしてしまったときは、`git submodule update --init`を実行してください。エディターはアドオンがなくてもビルドできますが、テストとGodotプロジェクトには必要です。

```sh
cargo run -p td_editor --release            # start the editor
cargo run -p td_editor --release -- map.tdm # open a map
```

VulkanかDX12かMetalに対応したGPUが必要で、LinuxではX11かWaylandのセッションが必要です。

Rustは1.95が最低限で、egui 0.36がそれを要求します。[rust-toolchain.toml](https://github.com/Paraxdev/TrenchDot/blob/main/rust-toolchain.toml)はCIが使うバージョンの1.98.1に固定しているので、最初のビルドでrustupがそれをインストールします。

## チェック { #checks }

クローンごとに一度、`./tools/fmt --install`を実行してください。ステージしたRustを整形するpre-commitフックが有効になります。

```sh
./tools/fmt --check
python tools/gen_licenses.py --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p td_editor --test e2e -- --ignored --test-threads=1
godot --headless --path godot --import                            # twice on a fresh checkout
godot --headless --path godot --script res://tests/run_tests.gd
godot --headless --path godot --script res://tests/build_showcase.gd
```

CIはこれらをすべて実行するので、プッシュの前に実行してください。

e2eテストは、本物のエディターを起動して、シミュレートした入力で操作します。そのためGPUとデスクトップセッションが必要で、`cargo test`は`--ignored`を付けない限りそれらを飛ばします。CIは代わりに、Linuxでソフトウェア版のVulkanを使って実行します。ウィンドウなしでLive Linkをテストするには、[Live Linkプロトコル](godot/live-link.md#testing-without-a-window)を参照してください。

> [!NOTE]
> `godot --headless --script`は何も描画せず(スクリーンショットや描画を使うチェックは真っ黒になります)、プロジェクトのオートロードも実行せず、スクリプト自体だけを実行します。テストは、ピクセルやオートロードの状態ではなく、データとノードに対して書いてください。

マップのビルド時間を計るには、`godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1`を実行します。その用途に`build_showcase.gd`は使わないでください。コミット済みのショーケースのシーンを上書きしてしまいます。

## gitでのマップファイル { #map-files-in-git }

`.tdm`のマップはバイナリです。[.tdmマップ形式](format/README.md)を参照してください。リポジトリの`.gitattributes`が、すでにそれらを印付けしています。差分を読みやすくするには、クローンごとに一度、[変更の確認](editor/reviewing.md#maps-in-git)にある`git config`の行を実行するだけです。

## 生成されるコンテンツ { #generated-content }

一部のファイルは、手書きではなくスクリプトが作ります。スクリプトが読むものを変えたら、再実行してください。

| コマンド | 再生成するもの |
| --- | --- |
| `cargo run -p td_formats --example demo_maps` | `godot/tests/maps`のテストマップと、`godot/demo/maps`の小さなデモマップ |
| `python tools/fetch_demo_textures.py` | デモのCC0の写真テクスチャ。[テクスチャサイズ](godot/materials.md#texture-size)を参照 |
| `python tools/fetch_demo_props.py` | `godot/models/polyhaven`にあるCC0のPoly Havenのプロップ。ガラスと炎のアルファマップも復元します。`--fix-alpha`は、それだけを復元します |
| `cargo run -p td_samples --bin showcase --release` | デモのBlockbenchモデル |
| `cargo run -p td_editor --example export_addon_fgd` | `builtin_entities.json`と`gameplay_entities.json`から作る、アドオンのコアFGDファイルとゲームプレイエンティティパックのテンプレート。アドオンのサブモジュールの中に書かれ、そのあとデモプロジェクトにインストール済みのパックのコピーにも書かれます |
| `python tools/gen_licenses.py` | [LICENSES.md](https://github.com/Paraxdev/TrenchDot/blob/main/LICENSES.md)のRustクレートの表 |

## Godotアドオン { #the-godot-addon }

アドオンには専用のリポジトリ[trenchdot_func](https://github.com/Paraxdev/trenchdot_func)があり、`godot/addons/func_godot`にチェックアウトされます。テストはデモプロジェクトが必要なので、ここの`godot/tests`にあります。アドオンを変更するには、次のようにします。

1. `godot/addons/func_godot`の中で変更をコミットして、trenchdot_funcにプッシュします。
2. 新しいサブモジュールのコミットを、ここで`git add godot/addons/func_godot`でステージし、それを必要とするTrenchDotの変更と一緒にコミットします。

アドオンを先にプッシュしてください。GitHubにないアドオンのコミットを指すTrenchDotのコミットは、CIでチェックアウトに失敗します。

`plugin.cfg`の`version`は、[Cargo.toml](https://github.com/Paraxdev/TrenchDot/blob/main/Cargo.toml)のワークスペースのバージョンと一致させます。2つは一緒に上げてください。プロジェクトのアドオンがこれと合わないと、エディターが警告します。

## 翻訳 { #translations }

エディターのUIは、英語、ドイツ語、フランス語、スペイン語、ポルトガル語、ロシア語、簡体字中国語、日本語で使えます。[言語](editor/interface.md#language)を参照してください。英語がソースで、ほかの7つは[crates/td_editor/locales](https://github.com/Paraxdev/TrenchDot/tree/main/crates/td_editor/locales)にあり、1言語につきJSONファイル1つで、エディターのビルド時に埋め込まれます。そこのREADMEに、形式、複数形、用語集があります。

コードでは、英語のテキストがキーです。リテラルは、保存する場所では`en!("...")`で、表示する場所では`tr(...)`、`trf!("... {n} ...", n = count)`、`tr_n!(count, "{n} brush", "{n} brushes")`で包み、同じキーをすべてのファイルに足します。エントリのないテキストは英語で表示されます。`crates/td_editor/src/mcp`の下のコード、ログの行、マップの内容はこれらを通りません。MCPの返答とエラーは英語のままです。`Msg`は英語のテキストを翻訳の隣に保持するので、ステータスバーは一方を、MCPはもう一方を読めます。

```sh
TD_L10N_TODO=/tmp/l10n cargo test -p td_editor --lib i18n
```

このテストは、コードが使うキーがファイルにないとき、またはコードがもう使わないキーがファイルに残っているときに失敗します。`TD_L10N_TODO`に既存のフォルダーを設定すると、足りないキーを入れたファイルを言語ごとに1つ書き出します。別のテストは、ソースの中で参照を飛ばしているUIテキストを探すので、ラベルに生のリテラルを渡すと失敗します。翻訳はAIが書いたもので確認されていません。ネイティブの方による修正は、その言語のファイルだけに触れます。e2eテストのように`--default-prefs`で起動したエディターは、常に英語です。

マニュアルにも翻訳があります。[マニュアルの翻訳](#translating-the-manual)を参照してください。ページはメニュー項目やボタンを、エディターが表示するとおりに書くので、カタログの値を変えたあとは`python3 tools/translations.py check`を実行してください。古い表現のままのページが一覧に出ます。

## ドキュメント { #documentation }

ドキュメントは`docs/`にあるMarkdownのページで、[Zensical](https://zensical.org)がそれをサイトにします。リポジトリのルートにある`zensical.toml`が設定を行い、`docs/SUMMARY.md`が目次です。そこにないページもビルドされますが、ナビゲーションには載りません。`docs/requirements.txt`はZensicalのバージョンを固定し、`docs/stylesheets/td.css`は色を、`docs/javascripts/td.js`はMCPスイッチと用語集カードを持ち、`docs/overrides`はテーマのテンプレートをいくつか置き換えます。

次のコマンドは、リポジトリのルートで実行します。

```sh
python3 -m venv .venv                  # once
. .venv/bin/activate                   # .venv\Scripts\activate on Windows
pip install -r docs/requirements.txt   # once, and again when the pin changes
zensical serve                         # live preview of the English site on http://localhost:8000/TrenchDot/
python3 tools/translations.py check    # the translated pages against their English source
python3 tools/build_site.py            # the build CI runs, English at the root of _site and a folder per language
python3 tools/test_docs.py             # tests of the three tools and of the built site
python3 tools/check_site.py            # the link check CI runs on _site
```

`build_site.py`は、英語に対して1回、各言語に対して1回、`zensical build --clean --strict`を実行します。そのため、どの言語にも存在しないページや見出しへのリンクがあると失敗します。`SUMMARY.md`の項目の書き間違い、画像の欠け、ナビゲーションから漏れたページはここをすり抜けますが、`tools/check_site.py`が全言語で見つけます。一部の言語だけをビルドするには、言語を指定します。`python3 tools/build_site.py de ja`のようにします。

見出しのidは、見出しのテキストを小文字にし、スペースをハイフンにして、ほかの句読点を取り除いたものです。なので`## path_corner`は`#path_corner`になります。[help.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/help.rs)にあるエディターのヘルプリンクはこれらのidを指していて、テストがそれを確認します。見出しの名前を変えるときは、リンクと一緒に変えてください。

注記は、GitHub形式のアラートです。`> [!NOTE]`、`> [!TIP]`、`> [!WARNING]`を単独の行に書き、本文は下の引用行に書きます。そうすればGitHubでも表示されます。番号付きの手順に属するもの、つまり2つ目の段落、画像、コードブロックは、4スペースでインデントしてください。しないと、リストから外れてしまいます。

ページ用の画像は`docs/assets/<page>/`に置きます。

はじめての方が知らなそうな用語は、`GLOSSARY.md`にエントリを作ります。`## term`の見出しと、1段落の定義です。すると、各ページでその用語が最初に出てくる場所がリンクになり、カーソルを重ねると定義が表示されます。コード、リンク、見出しは対象外です。

`.github/workflows/pages.yml`は、`docs/`、`zensical.toml`、ドキュメントのツール、エディターのカタログに触れるプルリクエストのサイトをビルドし、`main`に入ったらGitHub Pagesに公開します。

### マニュアルの翻訳 { #translating-the-manual }

サイトは、エディターの言語であるドイツ語、フランス語、スペイン語、ポルトガル語、ロシア語、簡体字中国語、日本語でもビルドされます。英語がソースで、ほかの言語はエディターのコードと同じ名前です。`de`、`fr`、`es`、`pt`、`ru`、`zh-Hans`、`ja`です。言語ごとのサイトは`/TrenchDot/<code>/`にあり、エディターはその言語で動いているとき、マニュアルをそこで開きます。翻訳はAIが書いたもので、ネイティブの方による確認はまだ行われていないので、翻訳済みのサイトにはそのことを伝えるバナーが出ます。バナーは読者が閉じられます。その言語に堪能な方の修正は歓迎します。

**どこに何があるか。** 翻訳したページは`docs/translations/<code>/`にあり、英語のページと同じ名前です。`docs/editor/brushes.md`なら`docs/translations/de/editor/brushes.md`です。ページの隣にある`site.toml`には、ページではないテキストを入れます。テーマの言語、サイトの説明、配色ボタンとMCPスイッチの名前、バナー、未翻訳ページ用の注記、アラートのタイトル、`[nav]`の表です。`[nav]`の表は、`SUMMARY.md`にある各英語のタイトルを翻訳に対応づけます。ないタイトルは英語で表示されます。言語の一覧は、`zensical.toml`の`[[project.extra.alternate]]`です。

**sourceの行。** 翻訳したページは、従っている英語のページを示すフロントマターで始まります。

```yaml
---
source: 0123456789abcdef0123456789abcdef01234567
---
```

`source`は、英語のページに対して`git hash-object docs/<page>`が出力する値です。一致している間は翻訳が最新で、ビルドはそれを使います。英語のページが変わると一致しなくなり、翻訳は古いものになって、ビルドは代わりに、タイトルのあとに短い注記を付けて英語のページを表示します。そのため、英語のページを編集してもCIは壊れず、翻訳の編集も要りません。誰かが更新するまで、ほかの言語のサイトで、そのページだけが英語になります。ページが欠けているときも同じようにフォールバックするので、すべてのページがすべての言語にあり、言語ボタンはいつも同じページに着きます。

**コマンド。** すべてリポジトリのルートで実行します。

| コマンド | 内容 |
| --- | --- |
| `python3 tools/translations.py status` | 古いページ、足りないページ、孤立したページ、エントリのないナビのタイトルを一覧にします。失敗することはありません。`--lang de`で1言語に絞り、`--diff`で各ページが翻訳されてから英語で何が変わったかを見られます。 |
| `python3 tools/translations.py prep --lang de editor/brushes.md` | 1ページ分に翻訳者が必要とするものを出力します。フロントマター、見出しのid、翻訳付きのラベル、用語集の単語、英語のテキストです。 |
| `python3 tools/translations.py check [--lang de] [page ...]` | 下のルールを破っている最新の翻訳があると失敗します。名前を挙げたページは状態にかかわらずチェックされるので、最新になる前にチェックできます。 |
| `python3 tools/translations.py stamp --lang de editor/brushes.md` | `source`を今の英語のページに合わせ、ほかには何も触りません。 |

**ページを翻訳する、または更新する。** 4つのステップです。

1. そのページに`prep`を実行します。古いページでは、古い`source`以降の英語の差分も表示されます。
2. `docs/translations/<code>/`にページを書きます。先頭には、`prep`が出力したフロントマターを置きます。英語の構造を1行ずつそのまま保ちます。同じ見出し、リスト、画像、リンク、アラート、`td-mcp`ブロックです。翻訳するのはテキスト、画像の代替テキスト、リンクテキストだけで、ほかは訳しません。言語ごとの口調は、下を参照してください。
3. そのページに`check`を実行し、報告されたものを直して、きれいになるまで繰り返します。
4. `python3 tools/build_site.py`、`python3 tools/check_site.py`、`python3 tools/translations.py check`を実行して、1言語分のページをまとめてコミットします。

`stamp`は、英語の変更が翻訳に何も要らないとき、たとえば誤字、リンク先の変更、同じことを言い換えただけのときにだけ使ってください。ページの中身は見ないので、先に英語の差分を読んでください。ページが言っていることが変わったときは、翻訳を編集する必要があり、新しい`source`もそれと一緒に入ります。

**口調。** どの言語も、ソフトウェアのドキュメントでそこに使われる形で、[はじめの一歩](getting-started.md)の気さくな口調を保ちます。

| 言語 | 呼びかけ |
| --- | --- |
| de | du、小文字 |
| fr | vous |
| es | tú、中立的なスペイン語で、複数形はカタログのREADMEにあるとおりustedes |
| pt | você、ブラジルのポルトガル語 |
| ru | вы、小文字 |
| zh-Hans | 你 |
| ja | です/ます形、人称代名詞なし |

**`check`が強制すること。** ルールはすべての最新の翻訳に当てはまり、`check`はCIでビルドの前に実行されます。

* 見出しのidは、英語のものを保ちます。タイトルも含め、すべての見出しが、英語の見出しのidを、同じ順序と同じ繰り返しの接尾辞(`{ #inputs_1 }`)で持ちます。たとえば`## Größe ändern { #reshape }`です。Zensicalはidを作るときにASCII以外の文字を落としますし、エディターのF1リンク、ページ間のリンク、用語集、言語ボタンはどれも、英語のidに頼っています。
* コードは英語のまま、バイト単位で同じです。フェンスで囲んだブロックはコメントごとコピーし、英語のページのインラインコードはすべて翻訳に現れます。エンティティのクラス名、キー、インプットとアウトプット、MCPのツール名、JSON、ファイル名、キーの名前(`Ctrl`、`F1`)、製品名(TrenchDot、Godot、MCP、Live Link)も訳しません。
* リンクと画像はリンク先を保ち、変わるのはリンクテキストと代替テキストだけです。スクリーンショットは英語のものを使い、翻訳したページは同じファイルを指すので、公開したサイトには各画像が1つだけあります。
* ラベルはエディターのものです。太字のスパン、または` > `を含む斜体のスパンで、すべての部分がカタログ`crates/td_editor/locales/<code>.json`のキーであるものは、カタログの書き方のまま、部分を` > `でつないで、翻訳したページに現れなければなりません。ラベルはエディターが表示するとおりに、活用させずに書き、その周りに文を組み立ててください。カタログにないラベル、たとえばGodotやアドオンのボタンは、英語のままです。
* アラート、`td-mcp`ブロック、その`## MCP`の見出しは、そのままの位置に残します。アラートのマーカーは、`> [!NOTE]`のように単独の行に書きます。翻訳したタイトルはビルドがその後ろに付けます。GDScriptのブロックは`gdscript`のタグを保ちます。
* em dash、en dash、スペース付きのハイフン、絵文字は使いません。英語のページのルールは、どの言語にも当てはまります。
* 中国語と日本語では、段落、リスト項目、表のセルをそれぞれ1行のソースにします。2つのCJK文字の間の改行は、ブラウザーではスペースとして表示されるので、`check`が指摘します。全角の句読点は問題ありませんが、2重のダッシュ文字のU+2014とU+2015はダッシュとして数えられ、使えません。

**用語集。** `GLOSSARY.md`も、ほかのページと同じように翻訳し、各用語は英語のidを保ちます。`## Aktivator { #activator }`のようにします。用語を活用させる言語は、ほかの形を`data-forms`に並べます。`{ #baked-lighting data-forms="gebackenen Beleuchtung|gebackener Beleuchtung" }`のようにすると、ホバーカードもそれらを見つけます。UIの名前である用語はカタログの単語を使い、`check`が比較します。翻訳したページでは、用語が最初に現れる場所で、用語集そのままの形を使います。それがカードへのリンクになるからです。英語の用語集に用語を足したら、できれば翻訳したほうにも足してください。それまでは、各言語の用語集は英語です。

## CI { #ci }

CIは、ドキュメントやMarkdownだけを変える`main`へのプッシュを飛ばします。それ以外のグリーンな`main`へのプッシュはすべて、[ローリングベータ](https://github.com/Paraxdev/TrenchDot/releases/tag/beta)を、新しいエディター、アドオン、デモ、コンテンツのダウンロードで置き換えます。

エディターは、アドオンとコンテンツをGitHubのリリースからだけダウンロードします。itch.ioは、同じファイルを`addon`、`nature-pack`、`demo-content`などの独自のチャンネルで受け取る販売ページです。ビルドはローリングベータだけなので、これらの通常のチャンネルを埋めるのはそれです。

| リリースのzip | インストールするもの |
| --- | --- |
| `func_godot-trenchdot-addon.zip` | *Godot > アドオンをインストールまたは更新…* |
| `trenchdot-nature-pack.zip` | *Godot > プロジェクトにコンテンツを追加…* |
| `trenchdot-demo-content.zip` | *Godot > プロジェクトにコンテンツを追加…* |

各zipが持つフォルダーは、[content.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/content.rs)と[addon_install.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/addon_install.rs)に一覧があり、`ci.yml`のzipのコマンドは、テストがそれらと照合しています。

エディターは、GitHub APIでzipを見つけます。CIは`TRENCHDOT_CHANNEL=beta`でビルドするので、まずローリングベータを探します。ローカルのビルドは、まず自分のバージョンにちなんだ名前のリリースを試し、なければベータにフォールバックします。公開前のzipを試すには、`TRENCHDOT_RELEASES_API`環境変数で、エディターを別のサーバーに向けます。

CIは、ベータごとに印も付けます。エディター用の`TRENCHDOT_BUILD`と、zipに入れるアドオンの`plugin.cfg`に、`0.1.0-beta.202611041532.d194b57`のように書きます。リリース番号、UTCのコミット時刻、コミットです。エディターは、これでアドオンが古いベータのものだと判断し、更新を提案します。ローカルのビルドは、素のバージョンのままです。

<div class="td-mcp" markdown>

## MCP

AIエージェントで貢献しますか。[AGENTS.md](https://github.com/Paraxdev/TrenchDot/blob/main/AGENTS.md)に、ルールと、プルリクエストが通すべきチェックがあります。

e2eテストは、[MCPサーバー](mcp.md)経由でエディターを操作し、`example_mcp_scripts_replay`は`examples/mcp`のすべてのスクリプトを再生します。

自分のエディターをシェルから操作するには、次のようにします。

```sh
trenchdot --mcp-http=7850 --default-prefs &
TRENCHDOT_MCP_URL=http://127.0.0.1:7850/mcp python3 tools/mcp_client.py call get_state
```

[トークン](mcp.md#the-http-token)を試すには、`TRENCHDOT_MCP_TOKEN`を両方に同じ値で設定します。

### ドキュメントの中のMCP { #mcp-in-the-docs }

ヘッダーの**MCP**スイッチは、既定ではオフで、MCPについてのものをすべて隠します。MCPに触れない人にも、ページが読みやすいままになります。ページには、MCPのテキストをすべて、一番最後の1つの`## MCP`セクションに、`td-mcp`ブロックの中に入れてください。

```html
<div class="td-mcp" markdown>

## MCP

What only MCP users need.

</div>
```

内容の前後の空行は残してください。GitHubは、それがあるときだけ、中のMarkdownを読みます。MCPだけについてのページは、`SUMMARY.md`の**MCP**の部分に置きます。そこもスイッチが隠します。

</div>
