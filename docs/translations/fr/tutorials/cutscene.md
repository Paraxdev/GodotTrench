---
source: 12dfe8079d66dd10bb5ca7191aa90e4b3c01fe5b
---

# Cinématique scriptée { #scripted-cutscene }

Le joueur entre dans une pièce, un panneau apparaît, un guide s'éloigne, un baril explose, une lampe s'allume et la
sortie s'ouvre. Rien de tout ça n'a besoin de code à soi : une entité minuteur déclenche une sortie par temps fort, et
chaque sortie est câblée à l'entité qui doit agir. Vous la trouvez sous le nom de carte `scripted_scene` dans le projet de
démo.

![La carte scripted_scene dans le panneau Logique : start_zone déclenche cutscene.start, les sorties step de cutscene vont vers le guide, le baril et l'indice, et la sortie broken du baril va vers hp_readout. La lampe et la porte de sortie sont plus bas, hors de l'image](../assets/cutscene/logic-graph.png)

## La chronologie { #the-timeline }

Toutes les entités d'ici viennent avec le pack d'entités Gameplay. Installez-le d'abord si votre projet ne l'a pas encore,
avec *Gameplay > Installer les entités Gameplay*.

1. Couvrir l'entrée

    Dessinez un brush sur l'entrée et faites-en un `trigger_once` nommé `start_zone` (clic droit, *Créer une entité brush >
    trigger_once*). Il déclenche `triggered` la première fois que le joueur entre, puis se désactive, donc la scène ne
    se joue qu'une fois.

2. Ajouter la chronologie

    Choisissez `logic_sequence` dans *Gameplay > Logique*, elle atterrit au curseur 3D. Nommez-la `cutscene`, réglez
    `steps` sur 5 et `interval` sur 1.5. Une fois démarrée, elle déclenche `step_1`, `step_2` et ainsi de suite, à 1,5
    seconde d'écart.

    ![Le menu Gameplay avec Logique ouvert et logic_sequence survolé sous Placer au curseur](../assets/cutscene/menu-logic-sequence.png)

3. La démarrer depuis le trigger

    Liez la sortie `triggered` de `start_zone` à `cutscene.start`, avec *Gameplay > Logique > Lier deux entités
    sélectionnées…* comme dans [Un bouton ouvre une porte](button-door.md).

4. Câbler chaque temps fort

    Liez chaque sortie step de `cutscene` à l'entité qui agit à ce moment, une paire à la fois. Faire glisser d'une
    broche de sortie vers une broche d'entrée dans le [panneau Logique](../gameplay/logic-graph.md) marche aussi.

| Sortie | Cible et entrée | Ce qui se passe |
| --- | --- | --- |
| `step_1` | `hint.show` | Un `game_text` affiche « Follow the guide » |
| `step_2` | `guide.start` | Un `npc_walker` part le long de sa chaîne de `path_corner`, une rangée de repères qui nomment chacun le suivant |
| `step_3` | `barrel.ignite` | Un `prop_physics` explosif brûle sa mèche, puis explose et se brise |
| `step_4` | `lamp.turn_on` | La lampe s'allume |
| `step_5` | `exit_door.open` | La porte de sortie coulisse vers le haut |

Sélectionnez `cutscene` et vous devriez voir les cinq sous *Sorties* dans son Inspecteur.

![L'Inspecteur de la logic_sequence nommée cutscene : steps 5, interval 1.5, et Sorties (E/S, 5) en commençant par step_1 vers hint.show](../assets/cutscene/inspector-sequence.png)

La séquence attend avant chaque étape, la première aussi, donc l'étape 1 se déclenche à 1,5 s et l'étape 5 à 7,5 s. Pour
un rythme inégal, réglez `times` sur une attente par étape, comme `0.5 2 1 1 3`.

## Lire la santé du joueur { #reading-the-players-health }

Certaines réactions demandent un peu de logique qu'aucune entrée intégrée ne couvre. Un `logic_script` exécute un court
extrait de GDScript quand il est déclenché. Liez la sortie `broken` du baril à `hp_readout.run`, un `logic_script` dont
`source` écrit la santé du joueur sur un `game_text` de HUD nommé `hpmsg` :

```gdscript
var players = io.find_targets(this, "!player", null)
if players.is_empty():
	return
var hp = int(players[0].health)
for label in io.find_targets(this, "hpmsg", null):
	label.set_text("HP " + str(hp))
	label.show()
```

Utilisez `!player` ici, pas `!activator`. `!activator` désigne celui qui a lancé la chaîne, et les sorties step et
`broken` ne portent personne. `!player` trouve toujours le nœud du groupe `player`. `broken` se déclenche une fois que
l'explosion a fait ses dégâts, donc la santé est déjà à jour.

## La jouer { #playing-it }

Le script de scène,
[godot/demo/scenes/scripted_scene.gd](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/scenes/scripted_scene.gd),
ne fait que construire la carte, faire apparaître le joueur sur l'`info_player_start` et ajouter un
`TrenchDotDebugOverlay`. Appuyez sur **F3** en jeu pour voir chaque sortie se déclencher. C'est le moyen le plus rapide de
repérer une étape câblée au mauvais nom.

<div class="td-mcp" markdown>

## MCP { #mcp }

La carte de démo est construite en pilotant l'éditeur via le [serveur MCP](../mcp.md). Les appels sont dans
[examples/mcp/scripted_scene.json](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/scripted_scene.json).

</div>
