---
source: d5a662c31724dba93b77bd318ab3a570c43e95b6
---

# Serveur MCP { #mcp-server }

L'éditeur intègre un serveur [MCP](https://modelcontextprotocol.io). Un agent comme Claude Code, ou un simple script
Python, peut s'en servir pour inspecter et modifier des cartes, prendre des captures d'écran et envoyer des entrées dans
l'interface.

Servez-vous-en pour laisser un assistant IA construire ou relire un niveau, pour scripter des modifications répétitives,
ou pour tester l'éditeur automatiquement. Le serveur est désactivé tant que vous ne le démarrez pas, et il n'écoute que
sur votre propre machine.

## Le démarrer { #starting-it }

| Transport | Comment le démarrer |
| --- | --- |
| HTTP | Lancez l'éditeur avec `--mcp-http` pour le port 7841, ou `--mcp-http=PORT` pour un autre port. Pour qu'il démarre à chaque fois, allez dans *Fichier > Préférences*, ouvrez *Avancé*, cochez *activer au démarrage* à côté de *Serveur MCP (HTTP, localhost)*, réglez le port à côté et redémarrez |
| stdio | Lancez avec `--mcp`. Le client démarre lui-même l'éditeur pour chaque session et lui parle par l'entrée et la sortie standard |

Choisissez HTTP quand vous gardez l'éditeur ouvert et y travaillez vous-même pendant qu'un agent vous aide. Choisissez
stdio quand le client doit posséder son éditeur, ou quand d'autres personnes utilisent le même ordinateur. Pour
connecter Claude Code à l'un ou l'autre :

```sh
claude mcp add --transport http trenchdot http://127.0.0.1:7841/mcp   # HTTP
claude mcp add trenchdot -- /path/to/trenchdot --mcp                 # stdio
```

Le `.mcp.json` du dépôt pointe déjà Claude Code vers le serveur HTTP. *Fichier > Préférences* montre la commande du
serveur qui tourne, et *Copier la commande* la met dans le presse-papiers.

Vous avez démarré l'éditeur sur un autre port ? Définissez `TRENCHDOT_MCP_URL` et
[tools/mcp_client.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_client.py) et
[tools/mcp_script.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_script.py) se connectent là à la place.

### Le jeton HTTP { #the-http-token }

N'importe quel programme de votre ordinateur peut joindre le serveur HTTP, et les autres personnes qui y sont connectées
aussi. Pour les tenir à l'écart, cochez *Exiger un jeton* sous *Avancé* dans *Fichier > Préférences*. Dès la requête
suivante, le serveur ne répond qu'aux requêtes qui envoient son jeton sous la forme `Authorization: Bearer <token>`. La
commande des Préférences inclut alors le jeton, utilisez-la à la place de `.mcp.json`. stdio n'a pas besoin de jeton,
seul le client qui a démarré l'éditeur peut lui parler.

L'éditeur fabrique un jeton aléatoire la première fois que vous en demandez un et le garde dans `mcp-token`, un fichier
que vous seul pouvez lire, à côté de ses réglages : `%APPDATA%\TrenchDot\data` sous Windows,
`~/.local/share/trenchdot` sous Linux et `~/Library/Application Support/TrenchDot` sous macOS. Une exécution avec
`--default-prefs` garde le sien dans `trenchdot-default-prefs.mcp-token` dans le dossier temporaire.

Pour choisir vous-même le jeton le temps d'une exécution, définissez `TRENCHDOT_MCP_TOKEN` ou lancez avec
`--mcp-token=<token>`, qui l'emporte sur la variable. L'un ou l'autre active le jeton quoi que disent les Préférences, et
*Exiger un jeton* apparaît coché et grisé. `tools/mcp_client.py` et `tools/mcp_script.py` n'envoient un jeton que si vous
leur en donnez un, avec `--token` ou `TRENCHDOT_MCP_TOKEN`.

## Désactiver MCP { #turning-mcp-off }

Pour tenir les agents et les scripts complètement à l'écart de l'éditeur, décochez *MCP* en haut de la partie MCP de
*Fichier > Préférences*. Le serveur HTTP se ferme aussitôt et coupe ses connexions ouvertes, un client sur stdio reçoit
une erreur à chaque requête, et les autres réglages MCP se grisent. Le réglage est enregistré tout de suite, donc chaque
éditeur que vous démarrez sans `--default-prefs` garde MCP désactivé jusqu'à ce que vous le cochiez de nouveau, ce qui
ramène le serveur HTTP sur son port.

![La partie MCP des Préférences : MCP coché, la ligne du serveur HTTP avec son port, et Exiger un jeton](assets/mcp/prefs-mcp.png)

Pour désactiver MCP le temps d'une exécution quoi que disent les Préférences, définissez `TRENCHDOT_DISABLE_MCP` à autre
chose que vide ou `0`. *MCP* apparaît alors décoché et grisé. Tant que MCP est désactivé, `--mcp` et `--mcp-http` ne
démarrent aucun serveur. L'éditeur s'ouvre quand même et sa ligne d'état dit que MCP est désactivé, et un client sur
stdio reçoit une erreur qui le dit au lieu d'attendre.

Un agent peut aussi désactiver MCP, avec `set_editor` et `"mcp": false`. Vous seul pouvez le réactiver.

## Outils { #tools }

| Domaine | À quoi il sert | Outils |
| --- | --- | --- |
| Inspecter | Lire la carte, l'état de l'éditeur, les définitions d'entités et la façon d'utiliser une entité depuis le code du jeu, sans rien changer | `get_state`, `summarize_map`, `list_nodes`, `get_node`, `changes_since`, `validate_map`, `get_game_config`, `code_reference` |
| Créer | Ajouter des brushes, des maillages, des terrains, des entités et des modèles importés, et faire du trim le long des arêtes de faces | `create_brush`, `create_mesh`, `create_terrain`, `create_entity`, `import_model`, `trim` |
| Modifier | Changer ce qui existe : clés d'entité, sélection, placement, faces, textures, terrain, calques et groupes, et worldspawn | `update_entity`, `select`, `transform`, `duplicate`, `set_face`, `mesh_edit`, `texture`, `terrain_edit`, `hierarchy`, `set_map_properties`, `bake_lighting` |
| Peinture et gameplay | Éparpiller des modèles, mélanger des matériaux, coller des pièces de la bibliothèque de prefabs, et transformer des brushes en portes, plateformes et boutons | `scatter`, `blend`, `prefab_library`, `random_fill`, `gameplay` |
| Éditeur | Lancer des actions de menu, ouvrir et enregistrer des fichiers, changer les réglages de l'éditeur, déplacer la caméra, prendre des captures, envoyer des entrées et vérifier où un joueur peut marcher | `run_action`, `map_file`, `open_project`, `set_editor`, `set_camera`, `screenshot`, `simulate_input`, `walkability` |
| Projet | Ajouter le pack d'entités Gameplay, les modèles de nature ou la démo au projet Godot ouvert | `project_content` |
| Scripts | Rejouer une liste enregistrée d'appels d'outils, voir [Scripts MCP](mcp-scripts.md) | `run_script` |

La description de chaque outil liste ses opérations et ses arguments, donc un agent les apprend directement auprès du
serveur. [tools/mcp_client.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_client.py) les affiche avec
`list` et appelle un outil depuis le shell :

```sh
python tools/mcp_client.py call screenshot '{"target":"3d","width":1280,"height":720}' --out shot.png
```

## Captures d'écran { #screenshots }

`screenshot` renvoie un PNG de toute la fenêtre de l'éditeur, ou d'une seule vue quand `target` vaut `3d`, `top`, `front`
ou `side`. Pour une vue, les autres arguments décident de ce que vous obtenez :

| Arguments | Résultat |
| --- | --- |
| Aucun | Toute la fenêtre de l'éditeur |
| `target` seul | Cette vue ancrée telle quelle, surcouches comprises |
| `width` et `height` | Rendu hors écran jusqu'à 4096 pixels, sans grille, boîtes d'entités, manipulateurs ni sélection |
| `width`, `height` et `"overlays": true` | Hors écran, avec les surcouches de l'éditeur |
| `"overlays": false` | Une capture propre à la taille ancrée |

### Ce que rend Godot { #what-godot-renders }

`"source": "godot"` demande à l'éditeur Godot de rendre la carte avec ses vraies lumières, son environnement, son
brouillard et son post-traitement. Vous récupérez ce PNG, plus les erreurs ou avertissements que Godot a journalisés en
la construisant. Pratique pour juger l'éclairage et les matériaux sans lancer le jeu.

```sh
python tools/mcp_client.py call screenshot '{"source":"godot","position":[0,256,512],"look_at":[0,64,0]}' --out godot.png
```

1. Enregistrez la carte dans le projet ouvert dans TrenchDot.
2. Ouvrez ce projet dans l'éditeur Godot, avec une scène dont le `FuncGodotMap` construit la carte.
3. Appelez `screenshot` avec `"source": "godot"`. Sans `position` ni `look_at`, il utilise la caméra de la vue 3D.

Sans [mode live](godot/live.md), Godot construit d'abord la carte telle qu'elle est montrée dans TrenchDot, comme
*Construire dans Godot*. `"build": false` capture la scène telle que Godot l'a. `width` et `height` valent 1280 par 720
par défaut, et `fov` celui de la vue 3D.

> [!NOTE]
> Godot a besoin d'une fenêtre pour faire le rendu, un Godot sans interface renvoie une erreur au lieu d'une image. La
> fenêtre peut être en arrière-plan ou réduite.

## Lire et relire une carte { #reading-and-reviewing-a-map }

| Étape | Outil |
| --- | --- |
| Avoir une vue d'ensemble | `summarize_map` liste les calques, les classes d'entités, les liens d'E/S et les matériaux, et les éventuels assets manquants |
| Trouver des nœuds | `list_nodes` filtre par `layer`, `material`, `property` (`{"model": "*crate*"}`) ou une `box` dans la carte |
| En regarder un | `get_node` renvoie un nœud. `compact: true` laisse de côté les sommets et les faces |
| Montrer ce que vous avez changé | `changes_since` compare avec le dernier enregistrement, un nombre d'`undo_steps` en arrière, ou un `file` de carte |
| Revenir en arrière | Chaque appel et chaque `run_script` est une étape d'annulation nommée `MCP: ...`. `run_action undo` avec `steps` en annule plusieurs |

Voir [Relire les modifications](editor/reviewing.md) pour garder les cartes dans git.

## Comportements à connaître { #behavior-worth-knowing }

| Sujet | Comportement |
| --- | --- |
| Navigateurs | Le serveur HTTP refuse toute requête avec un en-tête `Origin` et tout POST qui n'est pas `application/json`, donc les pages web ne peuvent pas l'atteindre, même celles servies depuis localhost |
| Erreurs | Un outil qui n'a pas pu faire son travail renvoie `isError`, y compris un `run_action` qui échoue |
| Boîtes de fichiers | Les actions qui en ouvrent une sont refusées, utilisez `map_file` avec un chemin. `run_action save` demande une carte qui a déjà un fichier |
| Onglets non enregistrés | `run_action close_tab` refuse les modifications non enregistrées sauf si `args.discard` vaut true. `args.close_others` ferme plutôt tous les onglets sauf l'actif |
| Chemins de fichiers | `res://` et les chemins relatifs dans `map_file`, `run_script`, `import_model`, `changes_since` et `validate_map` partent du projet ouvert, ou du dossier de travail de l'éditeur sans projet. `map_file open` et `open_tab` résolvent le chemin en chemin absolu avant d'ouvrir, pour qu'un `save` ultérieur ne puisse pas atterrir ailleurs |
| Réglages de l'éditeur | `transform` et `run_action rotate` et `flip` pivotent autour du centre de la sélection, aligné sur la grille seulement quand l'aimantation est active, et gardent les textures verrouillées tant que le verrouillage UV est actif. `transform` accepte `snap` et `uv_lock` pour le remplacer le temps d'un appel, `duplicate` accepte `uv_lock`. `run_action sculpt` et `blend` utilisent les réglages d'outil qu'une personne peut changer, `terrain_edit` utilise des valeurs fixes |
| Sélection | `select` refuse les ids inconnus et saute les nœuds masqués ou verrouillés, en les listant dans `skipped` |
| CSG | `csg_subtract`, `csg_merge`, `csg_intersect`, `csg_hollow` et `clip_apply` renvoient `replaced`, les anciens ids associés aux nouveaux |
| Délais | Les appels expirent après 120 s, sauf `run_script`, `project_content` et `bake_lighting` |
| Question de contenu | La question qu'un nouveau projet reçoit sur le contenu prêt à l'emploi se ferme au premier appel autre que `get_state`, `screenshot` et `simulate_input`, et ne se rouvre pas dans cette session |
| Variables de script | `$name/rest` (sans accolades) est une erreur quand `name` est une variable connue, car seul `$name` seul ou `${name}/rest` la développe. Écrivez `${project}/maps` |

## Notes sur les outils { #tool-notes }

| Outil | Note |
| --- | --- |
| `run_action` `copy`, `cut`, `paste` | Copy et cut renvoient le texte du presse-papiers, paste le prend dans `args.text` avec un `offset` ou un `origin` facultatif |
| `run_action` `clip_apply` | Prend un plan (`point` et `normal`, ou trois `points`) et `keep` : `front`, `back` ou `both` |
| `run_action` `create_decal` | Place un [decal](editor/decals.md), utilisez-le à la place de brushes fins |
| `create_brush` | `openings` perce des portes et des fenêtres, `{min, max, count, step, rows, row_step}` en répète une sous forme de grille |
| `create_mesh` `rotate` | `[x, y, z]` degrés dans l'ordre YXZ de Godot, donc `[0, 0, 90]` couche un cylindre le long de X |
| `transform` | Applique translate, rotate, flip et scale_to dans cet ordre, en une seule étape d'annulation. `rotate` tourne de 90 degrés autour de Y sauf indication contraire |
| `create_terrain` | `origin` est le coin minimum du terrain, contrairement aux autres outils. `center` le place plutôt autour d'un point |
| `create_brush`, `create_mesh` | Un faîte de pignon court le long de Z pour les brushes et le long de X pour les maillages. Les deux outils acceptent `ridge_x` et `ridge_z` |
| exports de `map_file` | `path` est le fichier écrit, gardé pour les anciens appelants, `exported` le nomme aussi et `map_path` est la carte ouverte |
| `terrain_edit` | `probe [x, z]` sans `op` ne fait que lire la hauteur. `clear_layer` demande `layer` |
| `validate_map` | Accepte comme entrée toute méthode ou propriété de la classe Godot ou du script de la cible. `coplanar_faces` signale les faces qui vont scintiller. Chaque problème a des `bounds`. `path` vérifie une carte enregistrée sans l'ouvrir, `project: true` vérifie toutes les cartes du projet |
| `run_action` `place_entities` | Prend `args.classnames`, et `at` et `normal` pour poser les entités sur une surface comme le fait un glisser depuis l'onglet Entités. Aucun classname est une erreur |
| glissers de `simulate_input` | Un glisser avec `"release": false` garde le bouton et ses modificateurs enfoncés après le retour de l'appel, donc une capture montre le glisser en cours, une carte à côté du curseur par exemple. Un événement `release` dans un appel ultérieur le lâche là où est le pointeur, ou à ses `x` et `y`, ou `world`, et un release avec un seul de `x` et `y` est une erreur. Le glisser et ses modificateurs restent enfoncés, y compris sur le vrai clavier, jusqu'à ce release, une action `new_map` ou un `close_tab` réussi, ou la désactivation de MCP, donc n'oubliez jamais le release. Les clics et les glissers sont refusés d'ici là |
| `run_action` `reload_materials` | Réanalyse tout de suite le dossier de textures, même si les nouveaux fichiers sont de toute façon trouvés tout seuls |
| `set_editor` `live_link_port` | Parle à un éditeur Godot dont le projet définit un autre `trenchdot/live_link_port` |
| `scatter` `preset` | Ajoute les modèles Blockbench intégrés dont un préréglage a besoin. Un préréglage glTF dans un projet sans le pack de nature est une erreur qui nomme `project_content` |
| `bake_lighting` | Précalcule comme [Précalculer l'éclairage](editor/light-baking.md) et attend la fin. Les options omises gardent ce avec quoi la carte a été précalculée en dernier. `bake: false` ne change que la `view`, `remove: true` supprime le précalcul. `out_of_date` dit si la carte a changé depuis |
| `project_content` | `install: "nature"` ou `"demo"` télécharge depuis la release et répond une fois les fichiers en place. `install: "gameplay"` copie le pack d'entités Gameplay depuis l'addon du projet, y compris dans les scripts. Sans arguments, il dit ce que le projet a |
