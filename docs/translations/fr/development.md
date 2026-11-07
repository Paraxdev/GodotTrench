---
source: d0d174df9061792086d11223ac4029e9d23f9d56
---

# Développement { #development }

Cette page est pour vous si vous voulez compiler l'éditeur depuis les sources, modifier l'addon ou écrire de la
documentation.

## Compiler { #building }

Clonez avec les sous-modules, l'addon Godot dans `godot/addons/func_godot` en est un :

```sh
git clone --recursive https://github.com/Paraxdev/TrenchDot.git
```

Vous avez déjà cloné sans cela ? Lancez `git submodule update --init`. L'éditeur se compile sans l'addon, mais les tests
et le projet Godot en ont besoin.

```sh
cargo run -p td_editor --release            # start the editor
cargo run -p td_editor --release -- map.tdm # open a map
```

Il vous faut un GPU avec Vulkan, DX12 ou Metal, et sous Linux une session X11 ou Wayland.

Rust 1.95 est le minimum, egui 0.36 l'exige.
[rust-toolchain.toml](https://github.com/Paraxdev/TrenchDot/blob/main/rust-toolchain.toml) fixe la 1.98.1, la version
qu'utilise la CI, donc rustup l'installe à votre première compilation.

## Vérifications { #checks }

Lancez `./tools/fmt --install` une fois par clone. Cela active un hook de pré-commit qui formate le Rust indexé.

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

La CI lance tout cela, lancez-le donc avant de pousser.

Les tests e2e lancent le vrai éditeur et le pilotent avec des entrées simulées. Il leur faut un GPU et une session de
bureau, donc `cargo test` les saute sauf si vous passez `--ignored`. La CI les lance sous Linux avec un Vulkan logiciel à
la place. Pour tester le live link sans fenêtre, voir [Protocole du live link](godot/live-link.md#testing-without-a-window).

> [!NOTE]
> `godot --headless --script` ne rend rien (les captures et les vérifications de rendu sortent noires) et ne lance pas
> les Autoloads du projet, seulement le script lui-même. Écrivez les tests sur les données et les nœuds, pas sur les
> pixels ni sur l'état d'un autoload.

Pour chronométrer les constructions de carte, lancez
`godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1`. N'utilisez pas
`build_showcase.gd` pour cela, il écrase les scènes showcase enregistrées.

## Les fichiers de carte dans git { #map-files-in-git }

Les cartes `.tdm` sont binaires, voir [le format de carte .tdm](format/README.md). Le `.gitattributes` du dépôt les
marque déjà. Pour avoir des diffs lisibles, il suffit de la ligne `git config` de
[Relire les modifications](editor/reviewing.md#maps-in-git), une fois par clone.

## Contenu généré { #generated-content }

Certains fichiers sont faits par un script, pas à la main. Quand vous changez ce que lit un script, relancez-le.

| Commande | Régénère |
| --- | --- |
| `cargo run -p td_formats --example demo_maps` | Les cartes de test dans `godot/tests/maps` et les petites cartes de démo dans `godot/demo/maps` |
| `python tools/fetch_demo_textures.py` | Les textures photo CC0 de la démo, voir [taille de texture](godot/materials.md#texture-size) |
| `python tools/fetch_demo_props.py` | Les props CC0 de Poly Haven dans `godot/models/polyhaven`, avec les cartes alpha du verre et des flammes rétablies. `--fix-alpha` ne fait que les rétablir |
| `cargo run -p td_samples --bin showcase --release` | Les modèles Blockbench de démo |
| `cargo run -p td_editor --example export_addon_fgd` | Les fichiers FGD principaux de l'addon et le modèle du pack d'entités Gameplay à partir de `builtin_entities.json` et `gameplay_entities.json`, écrits dans le sous-module de l'addon, puis la copie installée du pack dans le projet de démo |
| `python tools/gen_licenses.py` | Le tableau des crates Rust de [LICENSES.md](https://github.com/Paraxdev/TrenchDot/blob/main/LICENSES.md) |

## L'addon Godot { #the-godot-addon }

L'addon a son propre dépôt, [trenchdot_func](https://github.com/Paraxdev/trenchdot_func), récupéré dans
`godot/addons/func_godot`. Ses tests vivent ici dans `godot/tests`, puisqu'ils ont besoin du projet de démo. Pour modifier
l'addon :

1. Validez la modification dans `godot/addons/func_godot` et poussez-la vers trenchdot_func.
2. Indexez ici le nouveau commit du sous-module avec `git add godot/addons/func_godot` et validez-le avec les
   modifications de TrenchDot qui en ont besoin.

Poussez l'addon d'abord. Un commit TrenchDot qui pointe vers un commit d'addon que GitHub n'a pas ne s'extrait pas dans
la CI.

Le `version` de `plugin.cfg` correspond à la version du workspace de
[Cargo.toml](https://github.com/Paraxdev/TrenchDot/blob/main/Cargo.toml). Montez les deux ensemble, l'éditeur avertit
quand l'addon d'un projet ne correspond pas.

## Traductions { #translations }

L'interface de l'éditeur existe en anglais, allemand, français, espagnol, portugais, russe, chinois simplifié et
japonais, voir [Langue](editor/interface.md#language). L'anglais est la source, et les sept autres vivent dans
[crates/td_editor/locales](https://github.com/Paraxdev/TrenchDot/tree/main/crates/td_editor/locales), un fichier JSON
chacune, intégré à la compilation de l'éditeur. Le README de ce dossier donne le format, les formes plurielles et un
glossaire.

Dans le code, le texte anglais est la clé. Entourez un littéral de `en!("...")` à l'endroit où il est stocké et de
`tr(...)`, `trf!("... {n} ...", n = count)` ou `tr_n!(count, "{n} brush", "{n} brushes")` à l'endroit où il est dessiné,
et ajoutez la même clé à chaque fichier. Un texte sans entrée s'affiche en anglais. Le code sous
`crates/td_editor/src/mcp`, les lignes de journal et le contenu des cartes ne passent jamais par là, les réponses et les
erreurs MCP restent en anglais. Un `Msg` garde le texte anglais à côté de la traduction, pour que la barre d'état puisse
montrer l'un pendant que MCP lit l'autre.

```sh
TD_L10N_TODO=/tmp/l10n cargo test -p td_editor --lib i18n
```

Cette exécution échoue tant qu'un fichier n'a pas une clé que le code utilise ou garde encore une clé qu'il n'utilise plus.
Avec `TD_L10N_TODO` défini sur un dossier existant, elle écrit aussi un fichier par langue avec les clés manquantes. Un
autre test parcourt les sources à la recherche de texte d'interface qui saute la recherche, donc un littéral brut donné à
un label le fait échouer. Les traductions sont écrites par une IA et non vérifiées, la correction d'un locuteur natif ne
touche que le fichier de sa langue. L'éditeur lancé avec `--default-prefs`, comme le font les tests e2e, est toujours en
anglais.

Le manuel a aussi des traductions, voir [Traduire le manuel](#translating-the-manual). Ses pages nomment les éléments de
menu et les boutons comme l'éditeur les montre, donc après avoir changé une valeur dans un catalogue lancez
`python3 tools/translations.py check`. Il liste les pages qui utilisent encore l'ancienne formulation.

## Documentation { #documentation }

La documentation est faite des pages Markdown de `docs/`, et [Zensical](https://zensical.org) les transforme en site.
`zensical.toml` à la racine du dépôt le configure, et `docs/SUMMARY.md` est la table des matières. Une page absente de
là est quand même construite, elle n'a juste pas de place dans la navigation. `docs/requirements.txt` fixe Zensical,
`docs/stylesheets/td.css` contient les couleurs, `docs/javascripts/td.js` le commutateur MCP et les cartes du glossaire, et
`docs/overrides` remplace quelques modèles du thème.

Lancez ceci depuis la racine du dépôt :

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

`build_site.py` lance `zensical build --clean --strict` une fois pour l'anglais et une fois pour chaque langue, donc il
échoue sur un lien vers une page ou un titre qui n'existe dans aucune d'elles. Une entrée de `SUMMARY.md` mal tapée, une
image manquante ou une page oubliée dans la navigation passent à travers, `tools/check_site.py` les attrape dans chaque
langue. Pour ne construire que certaines langues, nommez-les : `python3 tools/build_site.py de ja`.

L'id d'un titre est son texte en minuscules, les espaces deviennent des tirets et les autres signes de ponctuation
disparaissent, donc `## path_corner` donne `#path_corner`. Les liens d'aide de l'éditeur dans
[help.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/help.rs) pointent vers ces ids, et un test
les vérifie, donc ne renommez un titre qu'avec ses liens.

Les encadrés sont des alertes à la GitHub : `> [!NOTE]`, `> [!TIP]` ou `> [!WARNING]` sur une ligne à part, avec le texte
sur les lignes citées en dessous. Ainsi elles s'affichent aussi sur GitHub. Tout ce qui appartient à une étape numérotée,
un deuxième paragraphe, une image ou un bloc de code, est indenté de 4 espaces, sinon cela sort de la liste.

Les images d'une page vont dans `docs/assets/<page>/`.

Si un nouveau venu ne connaîtrait pas un terme, donnez-lui une entrée dans `GLOSSARY.md` : un titre `## term` avec une
définition d'un paragraphe. Sa première utilisation sur chaque page renvoie alors vers elle et montre la définition au
survol. Le code, les liens et les titres sont laissés tels quels.

`.github/workflows/pages.yml` construit le site pour les pull requests qui touchent `docs/`, `zensical.toml`, les outils
de la doc ou les catalogues de l'éditeur, et le publie sur GitHub Pages une fois qu'elles arrivent sur `main`.

### Traduire le manuel { #translating-the-manual }

Le site est aussi construit en allemand, français, espagnol, portugais, russe, chinois simplifié et japonais, les langues
de l'éditeur. L'anglais est la source, et les autres langues portent les codes de l'éditeur : `de`, `fr`, `es`, `pt`,
`ru`, `zh-Hans` et `ja`. Le site d'une langue vit sous `/TrenchDot/<code>/`, et l'éditeur y ouvre le manuel quand il
tourne dans cette langue. Les traductions sont écrites par une IA et aucun locuteur natif ne les a vérifiées, donc chaque
site traduit le dit dans un bandeau que le lecteur peut fermer. La correction de quelqu'un qui maîtrise la langue est la
bienvenue.

**Où vivent les choses.** Une page traduite se trouve dans `docs/translations/<code>/` sous le nom de la page anglaise,
`docs/translations/de/editor/brushes.md` pour `docs/editor/brushes.md`. À côté des pages, `site.toml` contient les textes
qui ne sont pas des pages : la langue du thème, la description du site, les noms du bouton de schéma de couleurs et du
commutateur MCP, le bandeau, la note pour les pages non traduites, les titres des alertes et un tableau `[nav]`. Le
tableau `[nav]` associe chaque titre anglais de `SUMMARY.md` à sa traduction, un titre qui lui manque s'affiche en
anglais. La liste des langues est `[[project.extra.alternate]]` dans `zensical.toml`.

**La ligne source.** Une page traduite commence par un front matter qui nomme la page anglaise qu'elle suit :

```yaml
---
source: 0123456789abcdef0123456789abcdef01234567
---
```

`source` est ce qu'affiche `git hash-object docs/<page>` pour la page anglaise. Tant qu'il correspond, la traduction est à
jour et la construction l'utilise. Dès que la page anglaise change, il ne correspond plus, la traduction est périmée, et
la construction montre à la place la page anglaise avec une courte note après son titre. Donc modifier une page anglaise
ne casse jamais la CI et n'exige aucune modification de traduction, la page repasse seulement en anglais sur les autres
sites jusqu'à ce que quelqu'un la mette à jour. Une page manquante retombe de la même façon, donc chaque page existe dans
chaque langue et le bouton de langue arrive toujours sur la même page.

**Commandes.** Elles se lancent toutes depuis la racine du dépôt.

| Commande | Ce qu'elle fait |
| --- | --- |
| `python3 tools/translations.py status` | Liste les pages périmées, manquantes et orphelines et les titres de navigation sans entrée. Elle n'échoue jamais. Ajoutez `--lang de` pour une langue et `--diff` pour voir ce qui a changé en anglais depuis la traduction de chaque page. |
| `python3 tools/translations.py prep --lang de editor/brushes.md` | Affiche ce dont un traducteur a besoin pour une page : le front matter, les ids de titres, les labels avec leur traduction, les mots du glossaire et le texte anglais. |
| `python3 tools/translations.py check [--lang de] [page ...]` | Échoue sur une traduction à jour qui enfreint une règle ci-dessous. Les pages que vous nommez sont vérifiées quel que soit leur état, vous pouvez donc en vérifier une avant qu'elle soit à jour. |
| `python3 tools/translations.py stamp --lang de editor/brushes.md` | Règle `source` sur la page anglaise telle qu'elle est maintenant et ne touche à rien d'autre. |

**Traduire ou mettre à jour une page.** Quatre étapes :

1. Lancez `prep` pour la page. Pour une page périmée, il montre aussi le diff anglais depuis l'ancien `source`.
2. Écrivez la page dans `docs/translations/<code>/`, avec d'abord le front matter qu'a affiché `prep`. Gardez la
    structure anglaise ligne pour ligne : mêmes titres, listes, images, liens, alertes et blocs `td-mcp`. Traduisez le
    texte, le texte alternatif des images et le texte des liens, rien d'autre, dans la voix de la langue, voir
    ci-dessous.
3. Lancez `check` pour la page et corrigez ce qu'il signale jusqu'à ce qu'il soit propre.
4. Lancez `python3 tools/build_site.py`, `python3 tools/check_site.py` et `python3 tools/translations.py check`, puis
    validez ensemble les pages d'une langue.

N'utilisez `stamp` que quand la modification anglaise n'exige rien dans la traduction, comme une coquille, une cible de
lien changée ou une reformulation qui dit la même chose. Il ne regarde pas la page, donc lisez d'abord le diff anglais.
Un changement de ce que dit la page exige de modifier la traduction, et le nouveau `source` vient avec.

**Voix.** Chaque langue garde la voix décontractée de [Premiers pas](getting-started.md), sous la forme qu'emploient les
docs de logiciel là-bas :

| Langue | Adresse |
| --- | --- |
| de | du, en minuscules |
| fr | vous |
| es | tú, en espagnol neutre, ustedes au pluriel comme le dit le README du catalogue |
| pt | você, portugais brésilien |
| ru | вы, en minuscules |
| zh-Hans | 你 |
| ja | forme です/ます, sans pronom |

**Ce que `check` impose.** Les règles valent pour chaque traduction à jour, et `check` tourne avant la construction dans
la CI.

* Les ids de titres restent les ids anglais. Chaque titre, le titre de la page aussi, porte l'id de son titre anglais,
    `## Größe ändern { #reshape }`, dans le même ordre et avec les mêmes suffixes de répétition (`{ #inputs_1 }`).
    Zensical supprime les lettres hors ASCII quand il fabrique un id, et les liens F1 de l'éditeur, les liens entre
    pages, le glossaire et le bouton de langue s'appuient tous sur les ids anglais.
* Le code reste anglais et identique à l'octet près. Les blocs de code sont copiés avec leurs commentaires, et chaque
    extrait de code en ligne de la page anglaise apparaît dans la traduction. Les classnames d'entités, les clés, les
    entrées et sorties, les noms d'outils MCP, le JSON, les noms de fichiers, les noms de touches (`Ctrl`, `F1`) et les
    noms de produits (TrenchDot, Godot, MCP, Live Link) ne sont pas traduits non plus.
* Les liens et les images gardent leurs cibles, seuls le texte du lien et le texte alternatif changent. Les captures
    d'écran sont les anglaises, une page traduite pointe vers les mêmes fichiers, donc le site déployé en contient un
    seul exemplaire.
* Les labels sont ceux de l'éditeur. Un passage en gras, ou en italique contenant ` > `, dont toutes les parties sont des
    clés du catalogue `crates/td_editor/locales/<code>.json` doit apparaître dans la page traduite comme l'écrit le
    catalogue, les parties jointes par ` > `. Écrivez le label comme l'éditeur le montre, sans l'accorder, et construisez
    la phrase autour. Un label qui n'est pas dans le catalogue, comme un bouton de Godot ou de l'addon, reste en anglais.
* Les alertes, les blocs `td-mcp` et leur titre `## MCP` restent où ils sont. Écrivez le marqueur d'alerte seul sur sa
    ligne, `> [!NOTE]`, la construction met le titre traduit après lui. Les blocs GDScript gardent leur étiquette
    `gdscript`.
* Pas de tiret cadratin, de tiret demi-cadratin, de tiret espacé ni d'émoji, les règles des pages anglaises valent dans
    chaque langue.
* En chinois et en japonais, chaque paragraphe, élément de liste et cellule de tableau tient sur une seule ligne source.
    Un retour à la ligne entre deux caractères CJK s'affiche comme une espace dans le navigateur, donc `check` le
    signale. La ponctuation pleine chasse est permise, mais les doubles tirets U+2014 et U+2015 comptent comme des tirets
    et ne sont pas autorisés.

**Le glossaire.** `GLOSSARY.md` se traduit comme n'importe quelle page, et chaque terme garde son id anglais,
`## Aktivator { #activator }`. Une langue qui accorde un terme liste les autres formes dans `data-forms`,
`{ #baked-lighting data-forms="gebackenen Beleuchtung|gebackener Beleuchtung" }`, pour que la carte au survol les trouve
aussi. Un terme qui est un nom d'interface utilise le mot du catalogue, et `check` les compare. Une page traduite emploie
un terme sous la forme propre au glossaire là où il apparaît d'abord, c'est ce qui la relie à la carte. Quand vous
ajoutez un terme au glossaire anglais, ajoutez-le aux glossaires traduits si vous le pouvez, d'ici là le glossaire de
chaque langue est en anglais.

## CI { #ci }

La CI saute les pushs sur `main` qui ne changent que la doc ou du Markdown. Tout autre push vert sur `main` remplace la
[bêta glissante](https://github.com/Paraxdev/TrenchDot/releases/tag/beta) par de nouveaux téléchargements de l'éditeur,
de l'addon, de la démo et du contenu.

L'éditeur ne télécharge l'addon et le contenu que depuis les releases GitHub. itch.io est une vitrine qui reçoit les
mêmes fichiers sur ses propres canaux, `addon`, `nature-pack`, `demo-content` et ainsi de suite. La bêta glissante est la
seule version, donc elle remplit ces canaux simples.

| Zip dans la release | Installé par |
| --- | --- |
| `func_godot-trenchdot-addon.zip` | *Godot > Installer ou mettre à jour l'addon…* |
| `trenchdot-nature-pack.zip` | *Godot > Ajouter du contenu au projet…* |
| `trenchdot-demo-content.zip` | *Godot > Ajouter du contenu au projet…* |

Les dossiers que contient chaque zip sont listés dans
[content.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/content.rs) et
[addon_install.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/addon_install.rs), et des tests
vérifient les commandes zip de `ci.yml` par rapport à eux.

L'éditeur trouve les zips par l'API GitHub. La CI le compile avec `TRENCHDOT_CHANNEL=beta`, donc il cherche d'abord dans
la bêta glissante. Une compilation locale essaie d'abord une release nommée d'après sa propre version, puis retombe sur la
bêta. Pour essayer un zip avant sa publication, pointez l'éditeur vers un autre serveur avec la variable d'environnement
`TRENCHDOT_RELEASES_API`.

La CI estampille aussi chaque bêta, dans `TRENCHDOT_BUILD` pour l'éditeur et dans le `plugin.cfg` de l'addon zippé, sous la
forme `0.1.0-beta.202611041532.d194b57` : le numéro de release, l'heure UTC du commit et le commit. C'est ainsi que
l'éditeur reconnaît un addon d'une bêta plus ancienne et propose la mise à jour. Une compilation locale garde la version
simple.

<div class="td-mcp" markdown>

## MCP { #mcp }

Vous contribuez avec un agent IA ? [AGENTS.md](https://github.com/Paraxdev/TrenchDot/blob/main/AGENTS.md) liste les règles
et les vérifications qu'une pull request doit passer.

Les tests e2e pilotent l'éditeur via le [serveur MCP](mcp.md), et `example_mcp_scripts_replay` rejoue chaque script de
`examples/mcp`.

Pour piloter votre propre éditeur depuis le shell :

```sh
trenchdot --mcp-http=7850 --default-prefs &
TRENCHDOT_MCP_URL=http://127.0.0.1:7850/mcp python3 tools/mcp_client.py call get_state
```

Définissez `TRENCHDOT_MCP_TOKEN` à la même valeur des deux côtés pour l'essayer avec un [jeton](mcp.md#the-http-token).

### MCP dans la doc { #mcp-in-the-docs }

Le commutateur **MCP** de l'en-tête, désactivé par défaut, masque tout ce qui concerne MCP pour que les pages restent
lisibles pour les gens qui n'y touchent jamais. Sur une page, mettez tout le texte MCP dans une seule section `## MCP` tout
à la fin, dans un bloc `td-mcp` :

```html
<div class="td-mcp" markdown>

## MCP

What only MCP users need.

</div>
```

Gardez les lignes vides autour du contenu, GitHub ne lit le Markdown à l'intérieur qu'avec elles. Les pages uniquement sur
MCP vont dans la partie **MCP** de `SUMMARY.md`, que le commutateur masque aussi.

</div>
