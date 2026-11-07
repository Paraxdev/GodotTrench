---
source: 755d17f52aebf0b32513b6e22a04abb368badeab
---

# Construire des cartes { #building-maps }

TrenchDot enregistre un fichier `.tdm` et n'écrit jamais de scènes Godot lui-même. Dans Godot, le nœud `FuncGodotMap` de
l'addon lit ce fichier et génère les maillages, les collisions et les nœuds d'entité. Le résultat est enregistré avec votre
scène comme n'importe quel nœud fait à la main, donc rien ne se reconstruit à l'exécution sauf si vous le demandez.

## Le nœud FuncGodotMap { #the-funcgodotmap-node }

1. Ajoutez le nœud

    Dans l'éditeur Godot, appuyez sur Ctrl+A dans votre scène, cherchez `FuncGodotMap` et ajoutez-le.

2. Pointez-le vers votre carte

    Le nœud étant sélectionné, réglez *Local Map File* dans l'Inspecteur de Godot sur le `.tdm` que vous avez enregistré.

3. Construisez-la

    Cliquez sur **Build Map** en haut de l'Inspecteur. Vous devez voir la carte dans la vue de Godot, et le nœud reçoit
    des enfants pour les maillages et les entités. Enregistrez la scène avec Ctrl+S pour les garder. **Clear Map** les
    supprime de nouveau.

Voici les réglages que vous voyez dans l'Inspecteur :

| Propriété | Ce qu'elle fait |
| --- | --- |
| Local Map File | Le chemin `res://` du `.tdm` à construire. C'est le choix normal. |
| Global Map File | Un fichier de carte en dehors du projet. Quand les deux sont réglés, celui-ci l'emporte. |
| Auto Rebuild On Save | Reconstruit le nœud chaque fois que TrenchDot enregistre cette carte, tant que sa scène est l'onglet actuel de l'éditeur Godot, voir [Reconstruire à l'enregistrement](live.md#rebuild-on-save). Activé par défaut. |
| Map Settings | D'où viennent les textures, les matériaux et les définitions d'entité, et la taille d'une unité de carte. Laissé vide, il utilise le réglage de projet `func_godot/default_map_settings`, voir [Configuration du projet](project-setup.md). Un nœud de carte ajouté avec un ancien addon y a stocké les réglages de carte par défaut de l'addon, et sa construction le signale dans la Sortie de Godot, videz le champ pour suivre le réglage de projet. |
| Use Baked Lighting | Éclaire la carte avec l'éclairage [précalculé dans l'éditeur](baked-lighting.md). Activé par défaut, décochez-le pour utiliser un autre lightmapper. |
| Warm Up Shaders | Compile les shaders de la carte pendant son chargement, pour que les premières secondes de jeu ne saccadent pas. Voir [Préchauffage des shaders](warm-up.md). |

L'addon ne construit que des fichiers `.tdm`. Pour utiliser un `.map` Quake ou un `.vmf` Hammer, ouvrez-le avec *Fichier >
Importer* dans TrenchDot et enregistrez-le d'abord en `.tdm`.

> [!WARNING]
> Une construction supprime tous les enfants du nœud de carte sauf les [overlays](overlays.md). Gardez vos propres nœuds à
> côté du nœud de carte, ou dans un overlay.

## Trois façons de construire { #three-ways-to-build }

| Déclencheur | Ce qui est construit | Quand l'utiliser |
| --- | --- | --- |
| Enregistrer dans TrenchDot | Chaque `FuncGodotMap` qui utilise ce fichier dans la scène ouverte dans l'éditeur Godot | La boucle de tous les jours, enregistrer et regarder le résultat dans Godot |
| *Godot > Construire dans Godot* | La carte telle qu'elle est dans TrenchDot en ce moment, enregistrée ou non, sans écrire le fichier | Pour essayer une modification non enregistrée dans Godot sans s'y engager |
| **Build Map** dans Godot | Le fichier enregistré | Quand TrenchDot est fermé, ou pour reconstruire une scène que vous venez d'ouvrir |

Les deux premières demandent un éditeur Godot avec ce projet ouvert, et elles ne touchent que la scène qui y est ouverte.
Une reconstruction marque cette scène comme modifiée, donc enregistrez-la dans Godot pour garder le résultat.

Tant que Godot n'a pas le projet ouvert, *Construire dans Godot* reste grisé. Il faut aussi que la carte ait été enregistrée
une fois dans le projet, pour que Godot sache quel `FuncGodotMap` l'utilise.

![Le menu Godot avec Construire dans Godot survolé](../assets/building/menu-build-in-godot.png)

## Construire depuis le code { #building-from-code }

Vous pouvez aussi construire une carte pendant que le jeu tourne, par exemple pour charger un niveau choisi dans un menu :

```gdscript
var map := FuncGodotMap.new()
map.map_settings = load("res://demo/demo_map_settings.tres")
map.local_map_file = "res://demo/maps/scripted_scene.tdm"
add_child(map)
map.build()
```

Le nœud émet `build_complete` quand il a fini, ou `build_failed` quand le fichier de carte est absent ou n'est pas un `.tdm`.

## Environnement et soleil { #environment-and-sun }

Les clés de ciel, de brouillard et de soleil de worldspawn construisent un `WorldEnvironment` et une lumière nommée `sun`
dans la carte, si bien qu'une carte paraît éclairée sans aucune configuration dans Godot. Appuyez sur Échap pour que rien ne
soit sélectionné, l'Inspecteur montre alors *Map (worldspawn)* avec ces clés listées. Sa clé `environment` choisit ce qui est
construit. Une clé que vous laissez non définie reste hors de la carte, donc une carte sans aucune d'elles ne construit pas
d'environnement du tout. Si un projet fait avec un ancien addon ne liste pas les clés, mettez l'addon à jour et
[exportez de nouveau la config de jeu](project-setup.md#the-game-config).

| `environment` | Construit |
| --- | --- |
| `1` | L'environnement et le soleil. C'est la valeur par défaut. |
| `sun_only` | Seulement le soleil, pour un jeu qui apporte son propre `WorldEnvironment` |
| `0` ou `none` | Rien |

Si la scène a déjà un `WorldEnvironment` ou un `DirectionalLight3D` à elle, la carte le garde et n'en ajoute aucun, donc un
jeu avec son propre style l'emporte toujours. Quand plusieurs cartes partagent une scène, la première construite l'éclaire.

## Rapport de construction { #build-report }

`build()` renvoie un Dictionary, aussi conservé dans `build_report`. Il vous dit combien de temps a pris chaque étape,
combien d'entités de chaque classe ont été construites et combien de sommets la carte compte. Le plus utile est la liste de
ce que la construction n'a pas trouvé, ce qui est en général une faute de frappe ou un fichier renommé ou jamais
commité :

| Clé | Ce qu'elle liste | Ce que vous voyez dans le jeu |
| --- | --- | --- |
| `missing_classes` | Des classes d'entité qu'aucune définition d'entité ne décrit | Une entité brush garde son maillage mais n'a ni script ni collision, une entité point est un `Marker3D` vide, voir [Entités qui ne font rien](entities-troubleshooting.md) |
| `missing_models` | Des chemins de modèle qui n'existent pas, avec les entités qui utilisent chacun | L'accessoire n'a pas de modèle |
| `missing_materials` | Des textures de face qui n'ont ni fichier de matériau ni image | Les faces montrent la texture de remplacement |
| `unknown_tool_textures` | Des textures d'outil comme `special/clip` que les réglages de carte nomment autrement | Les faces ne dessinent toujours rien, mais un brush origin ne fixe pas l'origine de son entité |
| `unresolved_targets` | Des sorties et des clés cibles dont la cible ne correspond à aucun targetname dans la carte ou ses overlays | La sortie tire dans le vide |

Pour l'afficher après chaque construction, cochez *Print Build Report* sous *Build Flags* sur le nœud, ou ajoutez
`FuncGodotMap.BuildFlags.PRINT_REPORT` à `build_flags` dans le code. Un test ou une vérification de CI peut lire le
Dictionary à la place :

```gdscript
var report := map.build()
if not report["missing_models"].is_empty():
	push_error("missing models: %s" % [report["missing_models"]])
```

Chaque construction affiche aussi un avertissement dans la Sortie de Godot pour ce qu'elle n'a pas trouvé : un pour toutes
les classes manquantes, et un pour chaque modèle, matériau ou texture d'outil manquant, quel que soit le nombre
d'accessoires qui partagent le chemin. Après un enregistrement, TrenchDot liste les mêmes avertissements dans son panneau
Problèmes.

> [!NOTE]
> Les overlays que vous ajoutez après `build()` ne sont pas là quand le rapport cherche les cibles, donc leurs targetnames
> apparaissent sous `unresolved_targets`.

## Cartes endommagées { #damaged-maps }

Une carte endommagée se construit quand même à partir des parties qui ont pu être lues, avec un avertissement `[TDM]` pour
chaque partie qui n'a pas pu l'être. Voir [le format de carte .tdm](../format/recovery.md).
