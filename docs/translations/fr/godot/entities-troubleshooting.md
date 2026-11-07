---
source: 1b2a2ceda473607c8db087c552aff86851d08ef2
---

# Entités qui ne font rien { #entities-that-do-nothing }

Quand Godot construit une entité sans sa définition, l'entité n'est qu'une coquille. Une entité brush comme une porte
garde son maillage, mais devient un simple `Node3D` sans script ni collision, et une entité point devient un `Marker3D`
vide. La même chose arrive quand une définition ne convient pas à son entité, par exemple une entité brush dont la
définition ne construit aucun corps physique. TrenchDot et Godot vous le disent tous les deux quand cela arrive, et cette
page explique chaque message.

## Où vous le voyez { #where-you-see-it }

1. La barre d'état

    Enregistrez la carte dans TrenchDot pendant que sa scène est ouverte dans Godot. Quand Godot a construit des classes
    pour lesquelles il n'a pas de définition, la barre d'état en bas à gauche les nomme, avec les réglages de carte avec
    lesquels Godot a construit :

    ![La barre d'état qui dit que Godot a reconstruit 1 nœud de carte, mais que res://crypt_settings.tres n'a pas de définition pour func_button et func_door, donc ils se construisent comme des nœuds simples](../assets/entities-troubleshooting/status-missing-classes.png)

2. Le panneau Problèmes

    Ouvrez l'onglet **Problèmes**, à côté d'Outliner et de Historique à gauche. Il liste chaque classe que Godot a
    construite comme un nœud simple, avec la correction, et les autres avertissements de cette construction commencent par
    « Godot : ». Cliquez sur une ligne pour sélectionner l'entité.

    ![Le panneau Problèmes avec deux avertissements, Godot a construit func_button et func_door comme des nœuds simples, chacun disant quel fichier ajouter à Base Fgd Files](../assets/entities-troubleshooting/issues-missing-classes.png)

    Avant que Godot ait construit la carte, le panneau Problèmes compare vos entités avec le FGD avec lequel Godot construit,
    que l'addon exporte avec la [config de jeu](project-setup.md#the-game-config). Une entité que ce FGD n'a pas reçoit
    l'avertissement « Godot construit avec ..., dont le FGD ... n'a pas ... ».

    Quand Godot affiche une scène qu'il a construite avant qu'une définition soit là, la barre d'état et le panneau
    Problèmes disent « Godot affiche ..., construit avec d'autres définitions d'entités ». Choisissez *Godot > Construire
    dans Godot*, ou appuyez sur **Build Map** dans Godot et enregistrez, pour la construire de nouveau. **Lancer le projet**
    dit la même chose à propos des scènes qu'il lance.

3. Le panneau Output de Godot

    Appuyez sur **Build Map** sur le nœud `FuncGodotMap` dans Godot. Le panneau Output en bas de l'éditeur Godot montre un
    avertissement `[TrenchDot]` pour les classes sans définition et un pour chaque définition qui ne convient pas. Cochez
    *Print Build Report* sous *Build Flags* sur le nœud pour obtenir tout le [rapport de construction](building.md#build-report).

## Entités sans définition { #entities-without-a-definition }

| Le message dit | Pourquoi | Faites ceci |
| --- | --- | --- |
| Installez le pack d'entités Gameplay | Les portes, boutons, triggers, accessoires et spawners viennent avec le pack, et le projet ne l'a pas | Choisissez *Gameplay > Installer les entités Gameplay* dans TrenchDot, voir [Configuration du projet](project-setup.md#installing-and-updating-the-pack) |
| Votre FGD n'a pas le pack d'entités Gameplay, ajoutez `trenchdot_default_fgd.tres` à ses Base Fgd Files | Votre FGD n'a pas de *Base Fgd Files* | Ajoutez `res://addons/func_godot/fgd/trenchdot_default_fgd.tres` aux *Base Fgd Files* du FGD que les réglages de carte nomment dans *Entity Fgd* |
| Le pack est installé, mais l'*Entity Fgd* est un FGD de l'addon | Les réglages de carte utilisent le FGD propre de FuncGodot ou un autre fichier de l'addon, qu'une mise à jour de l'addon remplace | Réglez *Entity Fgd* de ces réglages de carte sur le FGD que votre config de jeu utilise |
| Une classe à vous n'a pas de définition | Elle est dans le FGD de votre config de jeu, mais pas dans l'*Entity Fgd* des réglages de carte avec lesquels Godot construit | Réglez *Entity Fgd* de ces réglages de carte sur le FGD que votre config de jeu utilise, voir [Vos propres ressources](project-setup.md#your-own-resources) |
| Le nœud de carte utilise les réglages de carte par défaut de l'addon, pas ceux des Project Settings | Le nœud a été ajouté avec un ancien addon, qui stockait les réglages de carte par défaut dans la scène | Sélectionnez le nœud de carte dans Godot et videz *Map Settings* dans l'Inspecteur |
| La scène a été construite avec d'autres définitions d'entités | Elle a été construite avant que le pack ou une définition soit là, et pas depuis | Appuyez sur **Build Map** et enregistrez la scène. En [mode live](live.md#when-live-mode-rebuilds-the-whole-map), Godot le fait tout seul |

## Définitions qui ne conviennent pas { #definitions-that-do-not-fit }

Ces messages viennent du panneau Output de Godot, et après un enregistrement du panneau Problèmes comme avertissements
« Godot : ». Le panneau Problèmes signale aussi les deux derniers avant que vous enregistriez.

| Le message dit | Pourquoi | Faites ceci |
| --- | --- | --- |
| `func_crusher builds as a Marker3D, so its brushes have no collision` | La définition n'a pas de *Node Class*, et son script étend un nœud qui n'est pas un corps physique | Réglez *Node Class* sur `StaticBody3D`, `AnimatableBody3D` ou `Area3D`, ou faites étendre l'un d'eux au script |
| `func_crusher: Node Class AnimatedBody3D not found, building it as a Node3D` | Une faute de frappe, ou un `class_name` que Godot n'a pas encore enregistré | Corrigez le nom. Pour C#, voir [Entités C#](../gameplay/csharp.md) |
| `alarm: res://entities/alarm.gd has no @tool` | L'éditeur n'exécute que des scripts tool, donc les clés de la carte n'atteignent jamais le nœud | Ajoutez `@tool` en haut du script |
| `light is a point entity, so its brushes are dropped` | Une `FuncGodotFGDPointClass` est utilisée pour une entité brush | Faites de la définition une `FuncGodotFGDSolidClass` |
| `func_door is a brush entity without brushes` | Une entité brush sans rien à partir de quoi construire | Donnez-lui des brushes, ou faites de la définition une classe point |

## Entités qui se construisent mais ne réagissent pas { #entities-that-build-but-do-not-react }

| Ce que vous voyez | Pourquoi | Faites ceci |
| --- | --- | --- |
| Les triggers ne se déclenchent jamais, et l'Output de Godot dit que le trigger a ignoré votre joueur parce qu'aucun nœud n'est dans son `filter_group` | Par défaut, les triggers ne réagissent qu'aux corps du groupe `player` sur la couche physique 1 | Ajoutez votre joueur au groupe `player`. Sur une autre couche, ajoutez cette couche à *Trigger Collision Mask* dans les réglages de carte |
| Les portes et les boutons ne bougent jamais | Ils bougent quand le code de votre jeu appelle `use()` dessus | Voir [Laissez le joueur appuyer dessus](../tutorials/button-door.md#let-the-player-press-it) |
| `info_spawner` et `npc_walker` paraissent vides dans l'éditeur de Godot | Ils ne font apparaître ou ne chargent leur modèle que dans le jeu en cours | Voir [Spawners et chemins](../gameplay/entities/actors.md) |
| L'Output de Godot montre `Parse Error` pour un script dans `res://trenchdot/entities` | Godot a compilé un script du pack avant de connaître les classes qu'il utilise | Redémarrez Godot, puis appuyez sur **Build Map** et enregistrez |
