---
source: ac40f8ef73c25c546d11381f2a654f03d847b7aa
---

# Scripts MCP { #mcp-scripts }

Un script MCP est un fichier JSON avec une liste d'appels d'outils [MCP](mcp.md), et `run_script` les rejoue dans
l'ordre. Il garde la façon dont une carte a été construite sous forme d'étapes lisibles, vous pouvez donc reconstruire la
carte de zéro et la relire comme du texte dans une pull request, et les tests peuvent eux aussi le rejouer.

Les cartes de démonstration de [examples/mcp](https://github.com/Paraxdev/TrenchDot/tree/main/examples/mcp) sont
construites ainsi, et elles servent aussi d'exemples pour chaque outil.

| Script | Construit |
| --- | --- |
| `mountain_house.json` | Un chalet accroché à une falaise, avec un ascenseur de falaise et des forêts |
| `church_school.json` | Une église et une école où trois boutons ouvrent le portail de la cour |
| `lighthouse_forest.json` | Un phare avec un faisceau rotatif, un bateau sur un `func_train` et des feux follets qui apparaissent |
| `withered_city.json` | Une ville néon en ruine la nuit, avec de la foudre, de la lueur et du `ssr` |
| `night_district.json` | Une rue industrielle la nuit, avec un rideau métallique et un train de viaduc |
| `sea_island.json` | Le [tutoriel de l'île en mer](tutorials/sea-island.md) |
| `scripted_scene.json` | Le [tutoriel de la scène scriptée](tutorials/cutscene.md) |
| `logic_playground.json` | Une salle pour les [nœuds de math et de logique](gameplay/entities/logic.md) : des pièces, un score, une barrière sur la porte du coffre et une loterie de lumières |

## Les lancer { #running-them }

Démarrez l'éditeur avec le projet de démonstration et son serveur HTTP, voir [Le démarrer](mcp.md#starting-it), puis
rejouez un script avec `tools/mcp_script.py`. Si l'éditeur demande un [jeton](mcp.md#the-http-token), passez-le avec
`--token` ou `TRENCHDOT_MCP_TOKEN`. Sa commande `shot` enregistre une capture d'écran depuis la position de caméra que
vous lui donnez, et `--shade` choisit l'ombrage :

```sh
trenchdot --mcp-http --project godot                        # the editor with the demo project
python tools/mcp_script.py run examples/mcp/sea_island.json   # replay a script
python tools/mcp_script.py shot view.png --pos 4300,900,3300 --look 2200,700,1200 --shade lit
```

Les scripts enregistrent dans `godot/demo/maps/showcase`, sauf `scripted_scene.json` et `logic_playground.json`, qui
enregistrent dans `godot/demo/maps`.

Une exécution s'arrête à la première étape qui échoue, sauf si vous passez `--continue-on-error`, et `--url` dirige le
lanceur vers un éditeur sur un autre port. Chaque exécution forme une seule étape d'annulation dans chaque carte qu'elle
modifie, nommée d'après le `label` du script ou son fichier.

## Format d'un script { #script-format }

```json
{
  "format": "trenchdot-mcp-script",
  "label": "Hollow room",
  "vars": { "save_dir": "${project}/maps" },
  "steps": [
    { "note": "A hollow room." },
    { "tool": "map_file", "args": { "op": "new" } },
    { "tool": "create_brush", "args": { "min": [0, 0, 0], "max": [256, 128, 256] }, "save": "room" },
    { "tool": "select", "args": { "ids": "$room.ids" } },
    { "tool": "run_action", "args": { "action": "csg_hollow" } },
    { "tool": "map_file", "args": { "op": "save", "path": "${save_dir}/room.tdm" } }
  ]
}
```

`vars` définit des valeurs par défaut pour vos propres variables. Un objet `vars` passé à `run_script` les remplace, un
même script peut donc enregistrer à différents endroits, et les dossiers que vous passez ainsi sont des dossiers où le
script peut lire et écrire.

Une étape sans `tool` est un commentaire. Une étape avec `save` stocke son résultat sous ce nom, pour que les étapes
suivantes puissent utiliser les ids qu'elle a créés.

Les scripts se partagent comme des cartes, leurs étapes ne font donc que modifier et enregistrer la carte. Une étape ne
peut pas ouvrir un autre projet ni l'une de ses cartes, démarrer Godot, y construire, activer le mode live, désactiver
MCP, copier ou couper dans le presse-papiers, jeter des modifications non enregistrées ni fermer d'autres onglets.
Annuler et rétablir n'atteignent que ce que le script a fait lui-même, le travail que vous n'avez pas enregistré reste
donc dans chaque onglet. `screenshot`, `simulate_input`, `walkability` et `run_script` ne peuvent pas non plus
s'exécuter dans un script, pas plus que les installations de `project_content` autres que `gameplay`.

Chaque fichier qu'une étape lit ou écrit doit se trouver dans le projet ouvert, le dossier du script lui-même ou un
dossier que vous passez dans `vars`, même quand un lien à l'intérieur pointe ailleurs. Cela couvre les cartes qu'elle
ouvre, vérifie, compare, enregistre et exporte, les cartes, modèles, prefabs et textures qu'elle importe avec les
fichiers qu'ils nomment, et tout ce que les conversions, les imports, les exports et la bibliothèque de prefabs
écrivent. Les prefabs, fichiers de modèle, textures et instances Hammer qu'une carte ou un modèle nomme ailleurs sont
laissés de côté, les chemins réseau sont refusés avant que l'éditeur les regarde, et un enregistrement sans chemin
n'écrit que dans la carte qui était ouverte au démarrage du script.

La règle vaut aussi pour les fichiers de matériau, leurs images et les fichiers de hotspots, la bibliothèque de prefabs
et les préréglages de nature. Une texture dont les fichiers se trouvent hors de ces dossiers est lue comme manquante dans
un script, si bien que l'ajuster ou l'aligner revient à la taille de texture par défaut. Le résumé liste alors les
fichiers qu'il n'a pas pu lire dans `unread_textures`.

Une étape refusée apparaît dans les `errors` du résumé avec la raison. Le résumé compte les appels d'outils comme `steps`
et les commentaires comme `notes`.

Les chaînes dans `args` peuvent renvoyer à des résultats enregistrés et à quelques variables intégrées :

| Écrivez | Signifie |
| --- | --- |
| `"$name.path"` | La valeur JSON enregistrée. Les nombres indexent les tableaux, comme dans `$room.ids.0` |
| `"${name.path}"` | Son texte à l'intérieur d'une chaîne plus longue |
| `"$a.ids + $b.ids"` | Les deux réunis en un seul tableau. `["$a.ids", "$b.ids"]` marche aussi |
| `$last` | Le résultat de l'étape précédente |
| `$project` | Le dossier du projet Godot ouvert, une erreur si aucun n'est ouvert |
| `$script_dir` | Le dossier du fichier de script |
| `$$` | Un `$` littéral, si bien qu'un chemin de nœud s'écrit `"$$Player"` |

Un nom inconnu est une erreur. Un `$` qui ne peut pas commencer un nom, comme `"$5"`, reste tel quel. Les ids donnés sous
forme de texte, comme `"42"`, marchent partout où un id de nœud est attendu.

`$name/rest` sans accolades ne se développe pas, donc un `$project/maps` sans accolades est une erreur plutôt qu'un
chemin que personne n'a voulu écrire. Utilisez `"${project}/maps"`.

## Les ids après un CSG { #ids-after-csg }

Les étapes de CSG et de coupe remplacent les brushes qu'elles découpent par de nouveaux brushes qui ont de nouveaux ids.
Le lanceur réécrit les ids enregistrés par les étapes précédentes pour qu'ils suivent, si bien que `"$wall.ids"` désigne
toujours le mur entier après que vous y avez découpé une fenêtre, et qu'un brush supprimé en disparaît.

Seules les valeurs sous des clés qui portent des ids de nœud sont réécrites : `id`, `ids`, `selection`, `selected`,
`nodes`, `letters`, `copies`, `entity`, `group`, `parent`, `insert_parent`, `current_layer`, `open_groups`, `targets` et
les clés qui finissent par `_id` ou `_ids`. Les compteurs comme `moved` de `hierarchy reparent` restent tels quels,
utilisez plutôt son `ids`.

> [!TIP]
> Écrivez chaque réglage de brush dans les appels `scatter` et `blend`. Tout ce que vous omettez vient des réglages
> actuels du panneau Scatter ou de l'outil Mélange de l'éditeur.
