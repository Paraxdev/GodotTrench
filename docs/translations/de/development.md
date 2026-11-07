---
source: d0d174df9061792086d11223ac4029e9d23f9d56
---

# Entwicklung { #development }

Diese Seite ist für dich, wenn du den Editor aus dem Quellcode bauen, das Addon ändern oder Doku schreiben willst.

## Bauen { #building }

Klone mit Submodulen, das Godot-Addon in `godot/addons/func_godot` ist eins:

```sh
git clone --recursive https://github.com/Paraxdev/TrenchDot.git
```

Schon ohne geklont? Führe `git submodule update --init` aus. Der Editor baut ohne das Addon, aber die Tests und das Godot-Projekt brauchen es.

```sh
cargo run -p td_editor --release            # start the editor
cargo run -p td_editor --release -- map.tdm # open a map
```

Du brauchst eine GPU mit Vulkan, DX12 oder Metal, und unter Linux eine X11- oder Wayland-Sitzung.

Rust 1.95 ist das Minimum, egui 0.36 verlangt es.
[rust-toolchain.toml](https://github.com/Paraxdev/TrenchDot/blob/main/rust-toolchain.toml) pinnt 1.98.1, die Version, die CI nutzt, rustup installiert sie also bei deinem ersten Build.

## Prüfungen { #checks }

Führe `./tools/fmt --install` einmal pro Klon aus. Es schaltet einen Pre-Commit-Hook ein, der gestagtes Rust formatiert.

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

CI führt all das aus, lass es also vor dem Push laufen.

Die e2e-Tests starten den echten Editor und steuern ihn mit simulierter Eingabe. Das braucht eine GPU und eine Desktop-Sitzung, deshalb überspringt `cargo test` sie, wenn du nicht `--ignored` übergibst. CI führt sie stattdessen unter Linux mit Software-Vulkan aus. Um den Live Link ohne Fenster zu testen, siehe [Live-Link-Protokoll](godot/live-link.md#testing-without-a-window).

> [!NOTE]
> `godot --headless --script` rendert nichts (Screenshots und Render-Prüfungen kommen schwarz heraus) und führt keine Autoloads des Projekts aus, nur das Skript selbst. Schreibe Tests gegen Daten und Nodes, nicht gegen Pixel oder den Zustand von Autoloads.

Um Kartenbuilds zu messen, führe `godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1` aus.
Nimm dafür nicht `build_showcase.gd`, es überschreibt die eingecheckten Showcase-Szenen.

## Kartendateien in Git { #map-files-in-git }

`.tdm`-Karten sind binär, siehe [das .tdm-Kartenformat](format/README.md). Die `.gitattributes` des Repositorys markiert sie schon. Für lesbare Diffs brauchst du nur die Zeile `git config` aus [Änderungen prüfen](editor/reviewing.md#maps-in-git), einmal pro Klon.

## Generierte Inhalte { #generated-content }

Manche Dateien macht ein Skript, nicht die Hand. Änderst du, was ein Skript liest, führe es erneut aus.

| Befehl | Erzeugt neu |
| --- | --- |
| `cargo run -p td_formats --example demo_maps` | Die Testkarten in `godot/tests/maps` und die kleinen Demokarten in `godot/demo/maps` |
| `python tools/fetch_demo_textures.py` | Die CC0-Fototexturen der Demo, siehe [Texturgröße](godot/materials.md#texture-size) |
| `python tools/fetch_demo_props.py` | Die CC0-Props von Poly Haven in `godot/models/polyhaven`, mit wiederhergestellten Alphamasken von Glas und Flammen. `--fix-alpha` stellt nur diese wieder her |
| `cargo run -p td_samples --bin showcase --release` | Die Blockbench-Modelle der Demo |
| `cargo run -p td_editor --example export_addon_fgd` | Die Kern-FGD-Dateien des Addons und die Vorlage des Gameplay-Entitätenpakets aus `builtin_entities.json` und `gameplay_entities.json`, geschrieben in das Addon-Submodul, danach die installierte Kopie des Pakets im Demo-Projekt |
| `python tools/gen_licenses.py` | Die Tabelle der Rust-Crates in [LICENSES.md](https://github.com/Paraxdev/TrenchDot/blob/main/LICENSES.md) |

## Das Godot-Addon { #the-godot-addon }

Das Addon hat ein eigenes Repository, [trenchdot_func](https://github.com/Paraxdev/trenchdot_func), ausgecheckt in `godot/addons/func_godot`. Seine Tests liegen hier in `godot/tests`, denn sie brauchen das Demo-Projekt. So änderst du das Addon:

1. Committe die Änderung in `godot/addons/func_godot` und pushe sie nach trenchdot_func.
2. Stage den neuen Submodul-Commit hier mit `git add godot/addons/func_godot` und committe ihn mit den TrenchDot-Änderungen, die ihn brauchen.

Pushe das Addon zuerst. Ein TrenchDot-Commit, der auf einen Addon-Commit zeigt, den GitHub nicht hat, lässt sich in CI nicht auschecken.

Die `version` in `plugin.cfg` entspricht der Workspace-Version in [Cargo.toml](https://github.com/Paraxdev/TrenchDot/blob/main/Cargo.toml). Erhöhe beide zusammen, der Editor warnt, wenn das Addon eines Projekts nicht dazu passt.

## Übersetzungen { #translations }

Die Oberfläche des Editors gibt es auf Englisch, Deutsch, Französisch, Spanisch, Portugiesisch, Russisch, in vereinfachtem Chinesisch und auf Japanisch, siehe [Sprache](editor/interface.md#language). Englisch ist die Quelle, und die anderen sieben liegen in
[crates/td_editor/locales](https://github.com/Paraxdev/TrenchDot/tree/main/crates/td_editor/locales), je eine JSON-Datei, beim Bauen des Editors eingebettet. Die README dort hat das Format, die Pluralformen und ein Glossar.

Im Code ist der englische Text der Schlüssel. Setze ein Literal dort, wo es gespeichert wird, in `en!("...")` und dort, wo es gezeichnet wird, in `tr(...)`, `trf!("... {n} ...", n = count)` oder `tr_n!(count, "{n} brush", "{n} brushes")`, und füge denselben Schlüssel in jede Datei ein. Ein Text ohne Eintrag erscheint auf Englisch. Code unter `crates/td_editor/src/mcp`, Logzeilen und Karteninhalt laufen nie hindurch, MCP-Antworten und Fehler bleiben Englisch. Ein `Msg` hält den englischen Text neben der Übersetzung, damit die Statusleiste den einen zeigen kann, während MCP den anderen liest.

```sh
TD_L10N_TODO=/tmp/l10n cargo test -p td_editor --lib i18n
```

Dieser Lauf schlägt fehl, solange einer Datei ein Schlüssel fehlt, den der Code nutzt, oder sie noch einen hält, den er nicht mehr nutzt. Ist `TD_L10N_TODO` auf einen existierenden Ordner gesetzt, schreibt er außerdem pro Sprache eine Datei mit den fehlenden Schlüsseln. Ein weiterer Test durchsucht den Quellcode nach UI-Text, der die Nachschlagefunktion überspringt, ein rohes Literal, das an ein Label übergeben wird, lässt ihn also fehlschlagen. Die Übersetzungen schreibt eine KI und sie sind ungeprüft, eine Korrektur von einer Muttersprachlerin oder einem Muttersprachler berührt nur die Datei dieser Sprache. Der Editor, gestartet mit `--default-prefs`, wie es die e2e-Tests tun, ist immer auf Englisch.

Das Handbuch hat ebenfalls Übersetzungen, siehe [Das Handbuch übersetzen](#translating-the-manual). Seine Seiten nennen Menüpunkte und Knöpfe so, wie der Editor sie zeigt, führe also nach dem Ändern eines Werts in einem Katalog `python3 tools/translations.py check` aus. Es listet die Seiten auf, die noch die alte Formulierung nutzen.

## Dokumentation { #documentation }

Die Doku sind die Markdown-Seiten in `docs/`, und [Zensical](https://zensical.org) macht daraus die Seite. `zensical.toml` im Wurzelverzeichnis des Repositorys richtet es ein, und `docs/SUMMARY.md` ist das Inhaltsverzeichnis. Eine Seite, die dort fehlt, wird trotzdem gebaut, sie hat nur keinen Platz in der Navigation. `docs/requirements.txt` pinnt Zensical, `docs/stylesheets/td.css` hält die Farben, `docs/javascripts/td.js` den MCP-Schalter und die Glossarkarten, und `docs/overrides` ersetzt ein paar Theme-Vorlagen.

Führe das im Wurzelverzeichnis des Repositorys aus:

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

`build_site.py` führt `zensical build --clean --strict` einmal für Englisch und einmal für jede Sprache aus, es schlägt also bei einem Link auf eine Seite oder Überschrift fehl, die in keiner davon existiert. Ein falsch getippter Eintrag in `SUMMARY.md`, ein fehlendes Bild oder eine Seite, die in der Navigation fehlt, kommen durch, `tools/check_site.py` fängt sie in jeder Sprache ab. Um nur einige Sprachen zu bauen, nenne sie: `python3 tools/build_site.py de ja`.

Die ID einer Überschrift ist ihr Text in Kleinbuchstaben, Leerzeichen werden zu Bindestrichen und andere Satzzeichen entfallen, `## path_corner` ist also `#path_corner`. Die Hilfe-Links des Editors in
[help.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/help.rs) zeigen auf diese IDs, und ein Test prüft sie, benenne eine Überschrift also nur zusammen mit ihren Links um.

Hinweise sind Alerts im GitHub-Stil: `> [!NOTE]`, `> [!TIP]` oder `> [!WARNING]` in einer eigenen Zeile, mit dem Text in den zitierten Zeilen darunter. So werden sie auch auf GitHub dargestellt. Alles, was zu einem nummerierten Schritt gehört, ein zweiter Absatz, ein Bild oder ein Codeblock, wird um 4 Leerzeichen eingerückt, sonst fällt es aus der Liste.

Bilder für eine Seite kommen nach `docs/assets/<page>/`.

Wenn ein Neuling einen Begriff nicht kennt, gib ihm einen Eintrag in `GLOSSARY.md`: eine Überschrift `## term` mit einer Definition in einem Absatz. Seine erste Verwendung auf jeder Seite verlinkt dann darauf und zeigt die Definition beim Darüberfahren. Code, Links und Überschriften bleiben unberührt.

`.github/workflows/pages.yml` baut die Seite für Pull Requests, die `docs/`, `zensical.toml`, die Doku-Werkzeuge oder die Editor-Kataloge berühren, und veröffentlicht sie auf GitHub Pages, sobald sie `main` erreichen.

### Das Handbuch übersetzen { #translating-the-manual }

Die Seite wird auch auf Deutsch, Französisch, Spanisch, Portugiesisch, Russisch, in vereinfachtem Chinesisch und auf Japanisch gebaut, den Sprachen des Editors. Englisch ist die Quelle, und die anderen Sprachen heißen wie die Codes des Editors: `de`, `fr`, `es`, `pt`, `ru`, `zh-Hans` und `ja`. Die Seite einer Sprache liegt unter `/TrenchDot/<code>/`, und der Editor öffnet das Handbuch dort, wenn er in dieser Sprache läuft. Die Übersetzungen schreibt eine KI, und keine Muttersprachlerin und kein Muttersprachler hat sie geprüft, deshalb sagt jede übersetzte Seite das in einem Banner, das ein Leser schließen kann. Eine Korrektur von jemandem, der die Sprache beherrscht, ist willkommen.

**Wo was liegt.** Eine übersetzte Seite liegt in `docs/translations/<code>/` unter dem Namen der englischen, `docs/translations/de/editor/brushes.md` für `docs/editor/brushes.md`. Neben den Seiten hält `site.toml` die Texte, die keine Seiten sind: die Sprache des Themes, die Beschreibung der Seite, die Namen des Farbschema-Knopfs und des MCP-Schalters, das Banner, den Hinweis für nicht übersetzte Seiten, die Titel der Alerts und eine Tabelle `[nav]`. Die Tabelle `[nav]` bildet jeden englischen Titel in `SUMMARY.md` auf seine Übersetzung ab, ein Titel, der ihr fehlt, erscheint auf Englisch. Die Liste der Sprachen ist `[[project.extra.alternate]]` in `zensical.toml`.

**Die Quellzeile.** Eine übersetzte Seite beginnt mit Front Matter, das die englische Seite nennt, der sie folgt:

```yaml
---
source: 0123456789abcdef0123456789abcdef01234567
---
```

`source` ist, was `git hash-object docs/<page>` für die englische Seite ausgibt. Solange es passt, ist die Übersetzung aktuell und der Build nimmt sie. Sobald sich die englische Seite ändert, passt es nicht mehr, die Übersetzung ist veraltet, und der Build zeigt stattdessen die englische Seite mit einem kurzen Hinweis nach ihrem Titel. Eine Änderung an einer englischen Seite bricht CI also nie und braucht keine Änderung an der Übersetzung, die Seite wird auf den anderen Seiten nur Englisch, bis jemand sie aktualisiert. Eine fehlende Seite fällt genauso zurück, jede Seite existiert also in jeder Sprache, und der Sprachknopf landet immer auf derselben Seite.

**Befehle.** Alle laufen im Wurzelverzeichnis des Repositorys.

| Befehl | Was er tut |
| --- | --- |
| `python3 tools/translations.py status` | Listet veraltete, fehlende und verwaiste Seiten sowie Nav-Titel ohne Eintrag auf. Er schlägt nie fehl. Mit `--lang de` für eine Sprache und `--diff`, um zu sehen, was sich seit der Übersetzung jeder Seite im Englischen geändert hat. |
| `python3 tools/translations.py prep --lang de editor/brushes.md` | Gibt aus, was ein Übersetzer für eine Seite braucht: das Front Matter, die Überschrift-IDs, die Labels mit ihrer Übersetzung, die Glossarwörter und den englischen Text. |
| `python3 tools/translations.py check [--lang de] [page ...]` | Schlägt bei einer aktuellen Übersetzung fehl, die eine Regel unten verletzt. Seiten, die du nennst, werden unabhängig von ihrem Zustand geprüft, du kannst eine also prüfen, bevor sie aktuell ist. |
| `python3 tools/translations.py stamp --lang de editor/brushes.md` | Setzt `source` auf die englische Seite, wie sie jetzt ist, und fasst sonst nichts an. |

**Eine Seite übersetzen oder aktualisieren.** Vier Schritte:

1. Führe `prep` für die Seite aus. Bei einer veralteten Seite zeigt es auch den englischen Diff seit dem alten `source`.
2. Schreibe die Seite in `docs/translations/<code>/`, mit dem Front Matter, das `prep` ausgegeben hat, zuerst. Behalte die englische Struktur Zeile für Zeile: gleiche Überschriften, Listen, Bilder, Links, Alerts und `td-mcp`-Blöcke. Übersetze den Text, den Alt-Text der Bilder und den Linktext, sonst nichts, in der Stimme der Sprache, siehe unten.
3. Führe `check` für die Seite aus und behebe, was es meldet, bis es sauber ist.
4. Führe `python3 tools/build_site.py`, `python3 tools/check_site.py` und `python3 tools/translations.py check` aus und committe dann die Seiten einer Sprache zusammen.

Nutze `stamp` nur, wenn die englische Änderung nichts in der Übersetzung braucht, etwa ein Tippfehler, ein geändertes Linkziel oder eine Umformulierung, die dasselbe sagt. Es sieht sich die Seite nicht an, lies also zuerst den englischen Diff. Eine Änderung dessen, was die Seite sagt, braucht eine Bearbeitung der Übersetzung, und das neue `source` kommt mit ihr.

**Stimme.** Jede Sprache behält die lockere Stimme von [getting-started.md](getting-started.md), in der Form, die Software-Doku dort nutzt:

| Sprache | Anrede |
| --- | --- |
| de | du, kleingeschrieben |
| fr | vous |
| es | tú, im neutralen Spanisch, ustedes im Plural, wie es die README des Katalogs sagt |
| pt | você, brasilianisches Portugiesisch |
| ru | вы, kleingeschrieben |
| zh-Hans | 你 |
| ja | です/ます-Form, kein Pronomen |

**Was `check` durchsetzt.** Die Regeln gelten für jede aktuelle Übersetzung, und `check` läuft in CI vor dem Build.

* Überschrift-IDs bleiben die englischen. Jede Überschrift, auch der Titel, trägt die ID ihrer englischen Überschrift, `## Größe ändern { #reshape }`, in derselben Reihenfolge und mit denselben Wiederholungssuffixen (`{ #inputs_1 }`). Zensical lässt Buchstaben außerhalb von ASCII weg, wenn es eine ID macht, und die F1-Links des Editors, die Links zwischen Seiten, das Glossar und der Sprachknopf verlassen sich alle auf die englischen.
* Code bleibt Englisch und Byte für Byte gleich. Codeblöcke werden mit ihren Kommentaren kopiert, und jede Inline-Code-Spanne der englischen Seite steht in der Übersetzung. Entitäts-Classnames, Schlüssel, Inputs und Outputs, MCP-Werkzeugnamen, JSON, Dateinamen, Tastennamen (`Ctrl`, `F1`) und Produktnamen (TrenchDot, Godot, MCP, Live Link) werden ebenfalls nicht übersetzt.
* Links und Bilder behalten ihre Ziele, nur Linktext und Alt-Text ändern sich. Die Screenshots sind die englischen, eine übersetzte Seite zeigt auf dieselben Dateien, die veröffentlichte Seite hält also eine Kopie von jedem.
* Labels sind die des Editors. Eine fette Spanne oder eine kursive, die ` > ` enthält, deren Teile alle Schlüssel des Katalogs `crates/td_editor/locales/<code>.json` sind, muss in der übersetzten Seite so stehen, wie der Katalog sie schreibt, die Teile mit ` > ` verbunden. Schreibe das Label so, wie der Editor es zeigt, unflektiert, und baue den Satz darum. Ein Label, das nicht im Katalog steht, wie ein Godot- oder Addon-Knopf, bleibt Englisch.
* Alerts, `td-mcp`-Blöcke und ihre Überschrift `## MCP` bleiben, wo sie sind. Schreibe die Alert-Markierung allein in ihre Zeile, `> [!NOTE]`, der Build setzt den übersetzten Titel dahinter. GDScript-Blöcke behalten ihr Tag `gdscript`.
* Kein Geviertstrich, Halbgeviertstrich, Bindestrich mit Leerzeichen und kein Emoji, die Regeln der englischen Seiten gelten in jeder Sprache.
* Im Chinesischen und Japanischen ist jeder Absatz, jeder Listenpunkt und jede Tabellenzelle eine Quellzeile. Ein Zeilenumbruch zwischen zwei CJK-Zeichen wird im Browser zu einem Leerzeichen, deshalb meldet `check` ihn. Satzzeichen voller Breite sind in Ordnung, aber die doppelten Strichzeichen U+2014 und U+2015 zählen als Striche und sind nicht erlaubt.

**Das Glossar.** `GLOSSARY.md` wird wie jede Seite übersetzt, und jeder Begriff behält seine englische ID, `## Aktivator { #activator }`. Eine Sprache, die einen Begriff beugt, listet die anderen Formen in `data-forms`, `{ #baked-lighting data-forms="gebackenen Beleuchtung|gebackener Beleuchtung" }`, damit auch die Hover-Karte sie findet. Ein Begriff, der ein UI-Name ist, nutzt das Wort aus dem Katalog, und `check` vergleicht sie. Eine übersetzte Seite nutzt einen Begriff dort, wo er zum ersten Mal vorkommt, in der eigenen Form des Glossars, das verlinkt sie mit der Karte. Wenn du dem englischen Glossar einen Begriff hinzufügst, füge ihn den übersetzten hinzu, wenn du kannst, bis dahin ist das Glossar jeder Sprache Englisch.

## CI { #ci }

CI überspringt Pushes nach `main`, die nur Doku oder Markdown ändern. Jeder andere grüne Push nach `main` ersetzt die
[fortlaufende Beta](https://github.com/Paraxdev/TrenchDot/releases/tag/beta) durch frische Downloads von Editor, Addon, Demo und Inhalten.

Der Editor lädt Addon und Inhalte nur von den GitHub-Releases. itch.io ist ein Schaufenster, das dieselben Dateien auf eigenen Kanälen bekommt, `addon`, `nature-pack`, `demo-content` und so weiter. Die fortlaufende Beta ist der einzige Build, sie füllt also diese schlichten Kanäle.

| Zip im Release | Installiert durch |
| --- | --- |
| `func_godot-trenchdot-addon.zip` | *Godot > Addon installieren oder aktualisieren…* |
| `trenchdot-nature-pack.zip` | *Godot > Inhalt zum Projekt hinzufügen…* |
| `trenchdot-demo-content.zip` | *Godot > Inhalt zum Projekt hinzufügen…* |

Die Ordner, die jedes Zip enthält, stehen in
[content.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/content.rs) und
[addon_install.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/addon_install.rs), und Tests prüfen die Zip-Befehle in `ci.yml` dagegen.

Der Editor findet die Zips über die GitHub-API. CI baut ihn mit `TRENCHDOT_CHANNEL=beta`, er sucht also zuerst in der fortlaufenden Beta. Ein lokaler Build probiert zuerst ein Release mit dem Namen seiner eigenen Version und fällt dann auf die Beta zurück. Um ein Zip vor seiner Veröffentlichung auszuprobieren, richte den Editor mit der Umgebungsvariable `TRENCHDOT_RELEASES_API` auf einen anderen Server.

CI stempelt außerdem jede Beta, in `TRENCHDOT_BUILD` für den Editor und in der `plugin.cfg` des gezippten Addons, als `0.1.0-beta.202611041532.d194b57`: die Release-Nummer, die UTC-Zeit des Commits und der Commit. So erkennt der Editor ein Addon aus einer älteren Beta und bietet das Update an. Ein lokaler Build behält die schlichte Version.

<div class="td-mcp" markdown>

## MCP { #mcp }

Du trägst mit einem KI-Agenten bei? [AGENTS.md](https://github.com/Paraxdev/TrenchDot/blob/main/AGENTS.md) listet die Regeln und die Prüfungen auf, die ein Pull Request bestehen muss.

Die e2e-Tests steuern den Editor über den [MCP-Server](mcp.md), und `example_mcp_scripts_replay` spielt jedes Skript in `examples/mcp` ab.

So steuerst du einen eigenen Editor von der Shell aus:

```sh
trenchdot --mcp-http=7850 --default-prefs &
TRENCHDOT_MCP_URL=http://127.0.0.1:7850/mcp python3 tools/mcp_client.py call get_state
```

Setze `TRENCHDOT_MCP_TOKEN` für beide auf denselben Wert, um es mit einem [Token](mcp.md#the-http-token) auszuprobieren.

### MCP in der Doku { #mcp-in-the-docs }

Der Schalter **MCP** in der Kopfzeile, standardmäßig aus, blendet alles zu MCP aus, damit die Seiten für Leute lesbar bleiben, die es nie anfassen. Setze auf einer Seite allen MCP-Text in einen Abschnitt `## MCP` ganz am Ende, in einem `td-mcp`-Block:

```html
<div class="td-mcp" markdown>

## MCP

What only MCP users need.

</div>
```

Lass die Leerzeilen um den Inhalt stehen, GitHub liest das Markdown darin nur mit ihnen. Seiten nur über MCP gehören in den Teil **MCP** von `SUMMARY.md`, den der Schalter ebenfalls ausblendet.

</div>
