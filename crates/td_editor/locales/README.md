# Editor translations

Each file here holds the editor UI in one language: `de`, `fr`, `es`, `pt` (Brazilian Portuguese), `ru`, `zh-Hans`
(Simplified Chinese, mainland) and `ja`. English is the source and needs no file. The editor embeds these files when
it is built, so a change shows after the next build.

The translations are written by AI and no native speaker has checked them yet. A fix from someone fluent is welcome and
touches nothing but the file of that language.

## Format

A file is one JSON object, UTF-8 without BOM, LF line ends, two space indent, one key per line, raw Unicode rather
than `\u` escapes. New keys go at the end, the files are not sorted.

```json
{
  "Open Recent": "Zuletzt geöffnet",
  "Deleted {n} objects": "{n} Objekte gelöscht",
  "{n} brush": { "one": "{n} Brush", "other": "{n} Brushes" }
}
```

The key is the English text as the code has it. A key ending in `#name` (lowercase letters and `_`) is the same English
text in another sense, for example `"Open#verb"`, and English drops the suffix when it draws it. A value is the
translation. Anything a file leaves out shows in English.

A counted text has an object of plural forms instead, keyed by the English singular. Each language holds exactly its
own forms, by the CLDR integer rules:

| Language | Forms | When |
|---|---|---|
| de, es | `one`, `other` | `one` is 1 only, so it may leave out `{n}` |
| fr, pt | `one`, `other` | `one` is 0 and 1 |
| ru | `one`, `few`, `many` | `one` 1, 21, 31, `few` 2 to 4, 22 to 24, `many` the rest (0, 5 to 20, 25, 111) |
| zh-Hans, ja | `other` | every count |

## Rules

* Keep every `{name}` placeholder, move it wherever the sentence needs it. `{{` and `}}` are literal braces.
* Keep leading and trailing spaces and a trailing `…` as in English.
* Key names (Ctrl, Shift, Alt, Cmd, Esc, Enter, Tab, F1 to F12) and product names (TrenchDot, Godot, MCP, FGD, GDScript,
  C#, UV, `.tdm`, Blender) stay as in English.
* No em dashes, en dashes or a spaced hyphen as punctuation, no emoji. Join clauses with a comma or start a new
  sentence, as the English text does.
* Tooltips are one short sentence, about 100 characters in Latin and Cyrillic and about 60 in Chinese and Japanese.
  Shorten the sentence rather than the meaning. The tests stop a value longer than 1.5 times its English plus 10
  characters, and a Chinese or Japanese value longer than its English.
* Menu commands follow each language's habit: the infinitive in German, French, Spanish and Portuguese ("Karte
  öffnen", "Ouvrir la carte"), the infinitive or a noun in Russian, a short verb phrase or noun in Chinese and
  Japanese. Japanese sentences use the です/ます form. Spanish is neutral (ustedes, no vosotros).
* A concept Godot has takes the word the Godot editor uses in that language, since users come from Godot. A concept
  Godot lacks follows TrenchBroom and Hammer use, and the glossary below settles it.
* Numbers stay as the code formats them, with a dot as the decimal mark, so copied values paste back into fields.

## Glossary

"=" keeps the English word.

| English | de | fr | es | pt | ru | zh-Hans | ja |
|---|---|---|---|---|---|---|---|
| Map | Karte | Carte | Mapa | Mapa | Карта | 地图 | マップ |
| Brush | = | = | = | = | Браш | 笔刷 | ブラシ |
| Face | Fläche | Face | Cara | Face | Грань | 面 | 面 |
| Edge | Kante | Arête | Arista | Aresta | Ребро | 边 | エッジ |
| Vertex | = | Sommet | Vértice | Vértice | Вершина | 顶点 | 頂点 |
| Entity | Entität | Entité | Entidad | Entidade | Сущность | 实体 | エンティティ |
| Prefab | = | = | = | = | Префаб | 预制体 | プレハブ |
| Group | Gruppe | Groupe | Grupo | Grupo | Группа | 组 | グループ |
| Layer | Ebene | Calque | Capa | Camada | Слой | 图层 | レイヤー |
| Material | = | Matériau | = | = | Материал | 材质 | マテリアル |
| Texture | Textur | = | Textura | Textura | Текстура | 纹理 | テクスチャ |
| Mesh | = | Maillage | Malla | Malha | Меш | 网格 | メッシュ |
| Scene | Szene | Scène | Escena | Cena | Сцена | 场景 | シーン |
| Node | = | Nœud | Nodo | Nó | Узел | 节点 | ノード |
| Light | Licht | Lumière | Luz | Luz | Свет | 灯光 | ライト |
| Bake | Backen | Précalculer | Precalcular | Pré-calcular | Запечь | 烘焙 | ベイク |
| Inspector | Inspektor | Inspecteur | = | Inspetor | Инспектор | 检查器 | インスペクター |
| Outliner | = | = | = | = | Структура | 大纲 | アウトライナー |
| Viewport | = | Fenêtre d'affichage | = | = | Вьюпорт | 视口 | ビューポート |
| Grid | Raster | Grille | Cuadrícula | Grade | Сетка | 栅格 | グリッド |
| Snap | Einrasten | Aimanter | Ajustar | Encaixar | Привязка | 吸附 | スナップ |
| Selection | Auswahl | Sélection | Selección | Seleção | Выделение | 选择 | 選択 |
| Tool | Werkzeug | Outil | Herramienta | Ferramenta | Инструмент | 工具 | ツール |
| Clip | = | = | = | = | Отсечение | 裁切 | クリップ |
| Hollow | Aushöhlen | Évider | Vaciar | Esvaziar | Сделать полым | 挖空 | 中空化 |
| Cutter | = | = | = | = | Резак | 切割体 | カッター |
| Cordon | = | = | = | = | Кордон | 隔离框 | コードン |
| Hotspot | = | = | = | = | Хотспот | 热点 | ホットスポット |
| Trim | = | = | = | = | Трим | 饰条 | トリム |
| Scatter | = | = | = | = | Разброс | 散布 | 散布 |
| Terrain | = | = | Terreno | Terreno | Ландшафт | 地形 | 地形 |
| Displacement | = | = | = | = | Дисплейсмент | 置换 | ディスプレイスメント |
| Gizmo | = | = | = | = | Гизмо | 小工具 | ギズモ |
| Model | Modell | Modèle | Modelo | Modelo | Модель | 模型 | モデル |
| Pack | Paket | = | Paquete | Pacote | Пакет | 资源包 | パック |
| Project | Projekt | Projet | Proyecto | Projeto | Проект | 项目 | プロジェクト |
| Live Link | = | = | = | = | = | = | = |
| Logic graph | Logikgraph | Graphe logique | Grafo lógico | Grafo lógico | Логический граф | 逻辑图 | ロジックグラフ |
| Issues | Probleme | Problèmes | Problemas | Problemas | Проблемы | 问题 | 問題 |
| History | Verlauf | Historique | Historial | Histórico | История | 历史 | 履歴 |
| Command palette | Befehlspalette | Palette de commandes | Paleta de comandos | Paleta de comandos | Палитра команд | 命令面板 | コマンドパレット |
| Preferences | Einstellungen | Préférences | Preferencias | Preferências | Настройки | 偏好设置 | 環境設定 |
| Restricted Mode | Eingeschränkter Modus | Mode restreint | Modo restringido | Modo restrito | Ограниченный режим | 受限模式 | 制限モード |
| Undo | Rückgängig | Annuler | Deshacer | Desfazer | Отменить | 撤销 | 元に戻す |
| Redo | Wiederherstellen | Rétablir | Rehacer | Refazer | Повторить | 重做 | やり直し |
| Save | Speichern | Enregistrer | Guardar | Salvar | Сохранить | 保存 | 保存 |

## Missing keys

The tests in `crates/td_editor/src/i18n/tests.rs` read every key the code looks up and compare it with these files.
Run them with `TD_L10N_TODO` set to a folder to get one file per language with the keys it lacks, English values to
replace and plural keys already split into the language's forms:

```sh
TD_L10N_TODO=/tmp/l10n cargo test -p td_editor i18n
```

The same tests name keys a file still holds that the code no longer uses. Changing an English text in the code makes
its old key such an orphan in every file, translate the new text and drop the old key.
