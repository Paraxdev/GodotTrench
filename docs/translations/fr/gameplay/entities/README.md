---
source: 9167db67d391d56c5d5d5ba31e5f260c3c7d0f58
---

# Référence des entités { #entity-reference }

Les entités du pack d'entités Gameplay, groupées selon ce à quoi elles servent. Chaque fiche dit à quoi sert l'entité,
puis liste ses entrées (ce que les autres entités peuvent lui demander de faire), ses sorties (les événements que vous
pouvez relier plus loin) et ses clés (les réglages de l'Inspecteur). La façon dont les sorties se connectent aux entrées
est expliquée dans [Entrées et sorties](../io.md).

| Page | Ce que vous y trouvez |
| --- | --- |
| [Logique](logic.md) | Des aides invisibles qui décident quand les choses arrivent : relais, vérifications « seulement si », portes, bascules, tirages et cas au hasard, compteurs, minuteurs, un événement de départ de carte et des chronologies de cinématique |
| [Maths et valeurs](math.md) | Des aides invisibles qui stockent, calculent et comparent des nombres, comme un score qui ouvre une porte |
| [Scripts et appels](scripting.md) | Lancer un bout de GDScript, appeler le code de votre jeu, jouer une animation ou afficher ce que fait une chaîne |
| [Portes, mouvements et boutons](movers.md) | Des entités brush qui bougent : portes coulissantes et battantes, ascenseurs, trains et boutons |
| [Déclencheurs](triggers.md) | Des volumes invisibles qui réagissent quand un corps entre dedans, pour lancer des événements, blesser, téléporter, pousser ou faire apparaître |
| [Spawners et chemins](actors.md) | Faire apparaître des ennemis ou des objets à ramasser, destinations de téléportation, points de chemin et personnages de cinématique scriptés |
| [Props, lumières et effets](effects.md) | Props physiques cassables, modèles statiques, explosions, texte à l'écran, lumières commutables, son, particules et sondes de lumière |
| [Sections streamées](sections.md) | Repères et volumes de liaison pour jouer plusieurs cartes comme un seul monde, sans écran de chargement |

## Installer et personnaliser { #installing-and-customizing }

Le pack ne fait pas partie de l'addon lui-même. Appuyez sur **Installer les entités Gameplay** dans la fenêtre *Configurer
ce projet*, ou choisissez *Gameplay > Installer les entités Gameplay*, et ses définitions et ses scripts sont copiés dans
`res://trenchdot/entities`. Jusque-là, le panneau Entités ne liste que le socle : worldspawn, les aides `func_`,
`light`, `light_spot`, les sondes de lumière et de réflexion et `info_player_start`.

Les copies appartiennent à votre projet. Modifiez le script d'une entité dans `scripts/` ou ses clés et ses valeurs par
défaut dans `definitions/`, supprimez les entités dont vous ne vous servez pas, ou ajoutez les vôtres. Une mise à jour de
l'addon propose un pack plus récent mais n'écrase jamais un fichier que vous avez modifié. La
[Configuration du projet](../../godot/project-setup.md#core-entities-and-the-gameplay-entities-pack) explique les mises
à jour et ce qui se passe quand deux définitions partagent un classname.

## Lire les fiches { #reading-the-entries }

* Les arguments entre parenthèses sont ce qu'une entrée accepte ou ce qu'une sortie transmet, par exemple
  `trigger(activator)` reçoit le corps ou l'entité qui a lancé la chaîne.
* Toute entité qui a des entrées ou des sorties a une clé `targetname`, sauf `logic_auto`. C'est le nom que les autres
  entités utilisent pour l'atteindre.
* Une clé que vous ne réglez pas prend la valeur par défaut indiquée.
* Les distances sont en unités de carte, 32 par mètre. Les vitesses sont en mètres par seconde, et `omni_range`,
  `spot_range` et `max_distance` sont en mètres.
* Les listes d'entrées et de sorties sont ce que proposent les sélecteurs de l'éditeur. À l'exécution, n'importe quelle
  méthode du script de l'entité fonctionne comme entrée, le sélecteur n'est donc pas la liste complète.

Pour ajouter vos propres classes, voyez [Entités personnalisées en GDScript](../custom-entities.md).

## Ce qui n'est pas dans le pack { #not-in-the-pack }

**Le socle** vient avec l'addon et n'a pas besoin du pack. Worldspawn contient les réglages du soleil, de la lumière
ambiante, du ciel, du brouillard et du streaming, et l'Inspecteur l'affiche quand rien n'est sélectionné. Les classes
FuncGodot `func_geo`, `func_detail`, `func_detail_illusionary` et `func_illusionary` construisent de la géométrie simple.
`info_player_start` marque l'endroit où le joueur apparaît mais ne fait rien apparaître lui-même, c'est le code de votre
jeu qui y place le joueur. [`light` et `light_spot`](effects.md) sont les lumières commutables.

Les entités brush et les entités modèle ont une collision par défaut, voyez [Collision](collision.md) pour les clés qui
la changent.

**Le joueur** n'est pas une entité, c'est une scène de votre propre jeu. Le pack l'attend dans le groupe `player` et sur
la couche de collision 1, là où cherchent les déclencheurs, avec une méthode `take_damage(amount, source)` qu'appellent
les déclencheurs de dégâts et les explosions. Un joueur sur une autre couche a besoin de cette couche dans *Trigger
Collision Mask*, voyez [Déclencheurs](triggers.md#shared-inputs-outputs-and-keys). Voyez
[`godot/demo/player.gd`](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/player.gd) pour un joueur complet.
