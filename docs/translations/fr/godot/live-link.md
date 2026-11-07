---
source: c998bca32c1c4bd85f0c95739d78a13d4ede986a
---

# Protocole du live link { #live-link-protocol }

Le live link est la façon dont TrenchDot et l'éditeur Godot se parlent. L'addon fait tourner un petit serveur dans
l'éditeur Godot, et TrenchDot s'y connecte pour déclencher des reconstructions à l'enregistrement, diffuser les
modifications du [mode live](live.md) et montrer si Godot tourne.

Vous aurez rarement besoin de cette page, le lien marche tout seul une fois l'addon activé. Elle est là pour le jour où
vous écrivez votre propre outil qui pilote Godot de la même manière, ou voulez comprendre pourquoi la connexion échoue.
Si un enregistrement ne reconstruit rien, commencez plutôt par
[Reconstruire à l'enregistrement](live.md#rebuild-on-save), la barre d'état dit souvent pourquoi.

## Transport { #transport }

Du JSON délimité par des sauts de ligne sur TCP, à l'adresse `127.0.0.1:7842`. Chaque message a un `event` et un `seq`
facultatif, et reçoit exactement une ligne de réponse qui reprend le `seq` et porte `ok`. Les échecs répondent
`ok: false` avec une `error`. L'éditeur garde une seule connexion ouverte et envoie un battement `status` chaque seconde.

Pour utiliser un autre port, changez-le des deux côtés : `trenchdot/live_link_port` dans les réglages du projet Godot et
*Live Link Godot* dans *Fichier > Préférences* dans TrenchDot.

L'autoload de [hot reload](live.md#hot-reload-in-a-running-game) dans un jeu en cours écoute sur `127.0.0.1:7843` et ne
répond qu'à `map_saved`.

## Événements { #events }

Un client envoie ces événements. La colonne Réponse liste les champs que Godot renvoie en plus de `ok` et `seq`, ou ce
qu'il fait.

| Événement | Champs | Réponse |
| --- | --- | --- |
| `hello` | | `project` et `godot` (version), un test rapide que le lien répond |
| `status` | | `project`, `godot` (version), `pid`, `scene` (l'onglet de scène que Godot affiche) et `maps: [{path, epoch}]` pour elle |
| `map_saved` | `path` | `rebuilt`, le nombre de nœuds de carte reconstruits depuis le fichier, `scene`, `waiting`, et les `missing_classes`, `settings` et `warnings` de la construction |
| `build` | `path`, `text` | Comme `map_saved`, construit à partir de `text` au lieu du fichier |
| `focus` | | Met la fenêtre Godot au premier plan |
| `export_game_config` | | Écrit `trenchdot_game.json` |
| `scan` | | Lance une analyse des fichiers du projet. Une fois finie, chaque session live vérifie ses définitions d'entités |
| `play` | | Enregistre les scènes ouvertes et lance la scène principale, ou `ok: false` avec une `error` quand le projet n'en a pas |
| `live_begin` | `path`, `text`, `content` | `epoch` et `unresolved` |
| `live_resync` | `path`, `text` | `epoch` et `unresolved` |
| `live_delta` | `path`, `ops` | `epoch` et `unresolved`, ou `ok: false, resync: true` quand les opérations ne collent pas |
| `live_end` | `path`, `revert` | Avec `revert`, reconstruit depuis le fichier enregistré |
| `inspect` | `path`, `ids` | `nodes` avec nom, classe, nombre d'enfants et position par id de nœud de carte, plus `live` et `pending` |
| `capture` | `path`, `camera`, `width`, `height`, `text` | `png` (base64), `width`, `height` et `warnings`, les erreurs et avertissements journalisés pendant la construction |
| `walkable` | `path`, `agent`, `text` | `vertices` (liste à plat de x, y, z), `polygons`, `islands` (indices de polygones, le plus grand d'abord), `units_per_meter`, `agent` et `msec` |

`capture` rend le monde du premier nœud de carte qui utilise `path`. `camera` contient `position` et `forward` en unités
et axes de la carte, et `fov`, le champ de vision horizontal en degrés. Avec `text`, Godot construit d'abord cette carte,
sinon il attend qu'une session live ait rattrapé son retard. La réponse arrive après une douzaine d'images, et les autres
messages reçoivent leur réponse entre-temps.

`walkable` précalcule le maillage de navigation de toute la scène éditée avec `TrenchDotNav` pour `agent` (`radius`,
`height`, `max_climb`, `max_slope` en mètres et en degrés) et le renvoie dans les unités et les axes de la carte. Avec
`text`, Godot construit d'abord cette carte, sauf si elle a déjà été construite à partir du même texte.

Les chemins sont absolus, avec des barres obliques. Seuls les nœuds `FuncGodotMap` de la scène éditée, l'onglet de scène
que Godot affiche, avec *Auto Rebuild On Save* activé, participent. `scene` est son chemin `res://`, ou vide quand aucune
scène n'est ouverte.

`waiting` liste les autres onglets de scène ouverts qui utilisent la carte. Après un `map_saved`, ils construisent le
fichier quand leur onglet est affiché, après un `build` ils ne le font pas. Un `map_saved` pour un prefab reconstruit
aussi les cartes de la scène éditée qui l'instancient, directement ou par d'autres prefabs, et les compte dans `rebuilt`.

Ce que les constructions d'un `map_saved` ou `build` ont signalé arrive dans trois listes, chacune seulement quand elle
n'est pas vide : `missing_classes` contient les noms de classe triés qu'aucune définition ne décrivait, `settings` les
chemins des réglages de carte que les constructions ont utilisés, et `warnings` jusqu'à 20 avertissements d'une ligne du
rapport de construction. `unresolved` dans une réponse live liste les noms de classe triés que cette étape a construits
sans définition, lui aussi seulement s'il y en a.

`scan` convient aux fichiers écrits hors de Godot, comme un pack d'entités installé. Une session live dont la scène a été
construite avec d'autres définitions d'entités que celles des réglages de carte après l'analyse reconstruit sa carte. Un
addon sans `scan` ni `play` les traite comme un événement inconnu.

`text` est la carte entière en JSON, avec l'éclairage précalculé sous `lightmap` quand la carte a un bake, pour qu'une
construction live le garde. Cela ajoute environ 11 octets par texel d'atlas, soit environ 11 Mo pour un atlas de 1024 par
1024, donc l'éditeur n'envoie la carte entière qu'au début d'une session, après un nouveau bake ou après un gros
changement.

`content` est l'id de contenu dans le bloc END du fichier enregistré, pour qu'une scène construite depuis un fichier
inchangé ne soit pas reconstruite au début d'une session.

Quand le port est pris, d'habitude par un second éditeur Godot avec le même projet, l'addon prévient une fois dans le
panneau Output et retente le port toutes les trois secondes, il prend donc le relais quand l'autre éditeur se ferme.

## Opérations live { #live-ops }

| Opération | Champs | Sens |
| --- | --- | --- |
| `set` | `id`, `parent`, `index`, `node` | Insère ou remplace un nœud, dans la forme JSON du format `.tdm`, sans enfants |
| `remove` | `id` | Retire un nœud avec son sous-arbre, seul l'id retiré le plus haut est envoyé |
| `translate` | `ids`, `offset` | Déplace des nœuds, envoyé pendant un glisser à la place de la géométrie |
| `properties` | `properties` | Remplace les propriétés du worldspawn |

L'`epoch` d'une carte augmente chaque fois qu'elle est construite hors d'une session live ou que les nœuds `FuncGodotMap`
qui l'utilisent changent. Quand le `status` suivant montre une nouvelle epoch, recommencez avec `live_begin`.

Les nœuds générés portent l'id de leur nœud de carte dans la métadonnée `_td_id`, et les nœuds de calque et de groupe
dans `_td_group`.

## Tester sans fenêtre { #testing-without-a-window }

Le lien marche aussi avec un éditeur Godot sans fenêtre, ce qui convient aux tests automatisés. Ouvrez une scène qui
contient un `FuncGodotMap`, puis envoyez `inspect` au port 7842 pour voir ce que Godot a construit :

```sh
godot --headless --editor --path godot res://path/scene.tscn
```

(Note : sur un checkout tout neuf, lancez d'abord deux fois `godot --headless --path godot --import`. Une seule passe
n'enregistre que les classes de l'addon.)

Un Godot sans fenêtre ne dessine rien, donc `capture` y répond par une erreur. Testez les captures avec un éditeur avec
fenêtre.

Les tests de l'addon font cela dans
[`godot/tests/editor/live_link_tabs.gd`](https://github.com/Paraxdev/TrenchDot/blob/main/godot/tests/editor/live_link_tabs.gd),
qui ouvre, ferme, recharge et change d'onglets de scène dans un éditeur sans fenêtre et vérifie ce que le lien rapporte.

<div class="td-mcp" markdown>

## MCP { #mcp }

Le [serveur MCP](../mcp.md) laisse des agents et des scripts piloter l'éditeur, et certains de ses outils passent par le
live link. L'outil `screenshot` utilise `capture` quand on lui demande une image de Godot.

Pour des tests sans fenêtre qui modifient aussi la carte, démarrez TrenchDot avec son serveur MCP sur le même projet que
l'éditeur Godot sans fenêtre et modifiez la carte par MCP :

```sh
trenchdot --mcp-http --project godot
```

</div>
