---
source: 5a39bcf2beab2f567018f04963913fd06af5df16
---

# Overlays { #overlays }

Chaque construction remplace ce que la carte a généré, donc tout ce que vous ajoutez à la main sous le nœud de carte
disparaît au prochain enregistrement. Un overlay est l'endroit où va le contenu Godot qui appartient à la carte et doit
survivre à chaque reconstruction : un distributeur avec son propre script, un effet de particules réglé dans Godot, une
lampe avec un AnimationPlayer.

![La cour du quartier de nuit : guirlandes lumineuses, lucioles et une enseigne viennent de l'overlay, les lampes viennent de la carte](../assets/overlays/night-district-courtyard.jpg)

## En ajouter un { #adding-one }

1. Ajoutez le nœud overlay

    Dans Godot, ajoutez un nœud `TrenchDotOverlay` comme enfant direct du `FuncGodotMap`.

2. Placez vos nœuds dessous

    Les constructions, le mode live et le hot reload n'y touchent jamais, donc ses nœuds gardent leur état et leurs
    connexions de signaux et sont enregistrés avec votre scène.

(Pour un seul nœud, vous pouvez vous passer de l'overlay : un enfant direct de la carte dans le groupe `trenchdot_keep`
survit aussi aux constructions. Un overlay ailleurs dans la scène peut pointer vers sa carte avec *Map Path* à la place, et
suit alors la transformation de la carte.)

Le contenu d'un overlay utilise l'espace local de la carte en mètres. À 32 unités par mètre, un enfant à `(2, 0, -3)` se
trouve là où TrenchDot affiche `(64, 0, -96)`.

## Un overlay dans sa propre scène { #an-overlay-in-its-own-scene }

Un overlay peut aussi être une scène à part. Cela vaut le coup quand plusieurs scènes utilisent la même carte et doivent
partager l'overlay, quand une personne travaille sur la scène du niveau et une autre sur l'overlay, ou quand un script
génère la scène du niveau, comme dans la démo.

1. Créez une scène dont la racine est un `TrenchDotOverlay`.
2. Glissez le `.tscn` sur le nœud `FuncGodotMap`, pour qu'il soit instancié comme enfant direct de la carte.

Godot montre une scène instanciée comme un seul nœud sans flèche pour la déplier. Pour accéder à son contenu :

| Vous voulez | Faites ceci |
| --- | --- |
| Modifier l'overlay partout où il est utilisé | Cliquez sur l'icône *Open in Editor* à côté du nœud dans le dock Scène, ou double-cliquez sur le `.tscn` dans le dock Système de fichiers |
| Voir ou changer ses nœuds dans cette scène seulement | Faites un clic droit sur le nœud et activez *Editable Children* |

Ouverte seule, la scène de l'overlay affiche un avertissement sur sa racine parce qu'elle n'est pas sous une carte. C'est
normal. Les boîtes en pointillés dans TrenchDot se mettent à jour à la prochaine construction ou au prochain enregistrement
de la scène avec la carte, puisque c'est seulement là que l'overlay connaît sa carte.

## Câbler les overlays { #wiring-overlays }

Les nœuds d'overlay et les entités de la carte peuvent se déclencher mutuellement par les mêmes sorties et entrées que les
entités de la carte utilisent entre elles.

| Vous voulez | Faites ceci |
| --- | --- |
| Qu'une sortie de la carte atteigne un nœud d'overlay | Donnez au nœud un enfant `TrenchDotOverlayIO` et réglez son *Targetname* |
| Qu'un nœud d'overlay déclenche des entités de la carte | Ajoutez un enfant `TrenchDotOutput`, choisissez le signal dans *Output* et remplissez la cible, l'entrée et le paramètre |
| Que du contenu voyage sur une entité mobile | Placez-le sous un `TrenchDotAnchor` avec *Target* réglé sur le targetname de l'entité |

Une entrée envoyée à un `TrenchDotOverlayIO` appelle la méthode du nœud de ce nom, règle sa propriété de ce nom, ou exécute
`show`, `hide` ou `toggle`. Donc les nœuds sans script marchent aussi : l'entrée `emitting` avec le paramètre `true` démarre
un `GPUParticles3D`. Chaque entrée émet aussi `input_received(input, parameter, activator)`.

Un `TrenchDotOutput` écoute n'importe quel signal de son parent, donc un `Area3D` se déclenche sur `body_entered` sans code.
Pour vos propres événements, ajoutez un signal au script du parent et choisissez-le :

```gdscript
extends StaticBody3D

signal flipped_on(activator: Node)
signal flipped_off(activator: Node)

var on := false

func use(activator: Node = null) -> void:
	on = not on
	if on:
		flipped_on.emit(activator)
	else:
		flipped_off.emit(activator)
```

Une ancre garde le décalage où vous l'avez placée et suit l'entité à travers les reconstructions, les modifications live et
les mouvements, donc un panneau d'avertissement monte avec son rideau métallique.

> [!WARNING]
> Le câblage survit aux reconstructions, pas l'état. Les entités de la carte reviennent dans leur état de départ, donc
> connectez-vous au signal `map_rebuilt(map)` de l'overlay et renvoyez votre état, par exemple rallumez les lampes.

## Le voir dans la démo { #see-it-in-the-demo }

Le quartier de nuit du projet de démo a tout cela. Sa scène `res://demo/showcase/night_district.scn` instancie la scène
d'overlay `res://demo/overlays/night_district_overlay.tscn` sous sa carte, parce qu'un script reconstruit les scènes de
démonstration. Ouvrez la scène d'overlay et vous y trouverez un tableau électrique qui commande les lampes de la cour, des
guirlandes lumineuses qu'un déclencheur de la carte allume, et une enseigne qui voyage sur le rideau métallique du garage.
