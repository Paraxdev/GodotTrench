---
source: 19de62e149bf31bc7f0453a04737b8fbd5510db4
---

# Un bouton ouvre une porte { #button-opens-a-door }

Un bouton sur le mur qui fait coulisser une porte quand le joueur appuie dessus. C'est le plus petit morceau utile
d'E/S d'entité : le bouton déclenche une sortie appelée `pressed`, et une connexion en fait un appel de l'entrée `open`
de la porte.

> [!TIP]
> Si les sorties, les entrées et `targetname` sont nouveaux pour vous, lisez d'abord
> [Comment fonctionnent les E/S d'entité](../gameplay/io.md). Ça ne prend que quelques minutes et cette page s'appuie
> dessus.

## Câbler { #wire-it }

`func_door` et `func_button` viennent avec le pack d'entités Gameplay. Si votre projet ne l'a pas encore, utilisez
*Gameplay > Installer les entités Gameplay*, ou **Installer les entités Gameplay** dans la fenêtre *Configurer ce
projet* (*Godot > Installer ou mettre à jour l'addon…*).

1. Faire la porte

    Dessinez un brush de la taille d'une porte dans un encadrement, [Premiers pas](../getting-started.md#4-build-a-room)
    montre comment en percer un. Faites un clic droit sur le brush et choisissez *Créer une entité brush > func_door*.

    ![Le menu du clic droit sur un brush de porte orange, avec Créer une entité brush ouvert et func_door survolé](../assets/button-door/create-brush-entity-func-door.png)

2. La nommer

    Dans l'Inspecteur à droite, saisissez `main_door` dans `targetname`. Ce nom permet aux autres entités de retrouver la
    porte. Laissez `travel` à `0 64 0`, pour que la porte coulisse de 64 unités (2 m) vers le haut quand elle s'ouvre.

    ![L'Inspecteur de la func_door avec targetname main_door et travel 0 64 0](../assets/button-door/inspector-targetname.png)

3. Faire le bouton

    Dessinez un petit brush sur le mur à côté de la porte et transformez-le en `func_button` de la même façon.

4. Sélectionner les deux

    Cliquez sur le bouton, puis Ctrl+clic sur la porte, dans une vue ou dans l'Outliner à gauche. Des murs gênent ?
    Saisissez `func` dans la case Filtre en haut de l'Outliner, seules ces deux entités sont alors listées.

    ![L'Outliner filtré par func, avec func_door (main_door) et func_button tous deux sélectionnés](../assets/button-door/outliner-filter-func.png)

5. Les lier

    Allez dans *Gameplay > Logique > Lier deux entités sélectionnées…*, ou appuyez sur **Lier…** en haut de
    l'Inspecteur.

    ![Le menu Gameplay avec Logique ouvert et Lier deux entités sélectionnées… survolé](../assets/button-door/menu-link-two-selected.png)

    L'entité qui déclenche doit être à gauche, ici c'est le bouton. La boîte de dialogue met à gauche celle que vous avez
    faite en premier, c'est-à-dire la porte, alors appuyez sur le petit bouton d'échange entre les deux noms. Choisissez
    `pressed` sous *Quand* et `open` sous *appeler*, puis cliquez sur **Lier**.

    ![La fenêtre Lier des entités avec func_button à gauche, func_door (main_door) à droite, Quand pressed et appeler open](../assets/button-door/link-dialog.png)

La barre d'état en bas à gauche devrait maintenant afficher `pressed > main_door.open`. Lisez-le ainsi : « quand
pressed, appelle open sur main_door ».

![La barre d'état affichant pressed > main_door.open](../assets/button-door/status-linked.png)

Sélectionnez le bouton seul et vous trouverez la connexion sous *Sorties* dans son Inspecteur, où vous pourrez la
modifier ou la supprimer plus tard.

![La section Sorties de la func_button : sortie pressed, cible main_door, entrée open](../assets/button-door/inspector-button-output.png)

> [!TIP]
> *Gameplay > Portes et mobiles*, aussi dans la section *Gameplay* de l'Inspecteur, transforme le brush sélectionné en
> porte à charnière ou coulissante en un clic. Il nomme la porte, règle son ouverture et place devant elle un
> `trigger_multiple` qui ouvre la porte quand le joueur s'approche. Supprimez ce trigger si seul le bouton doit ouvrir la
> porte.
>
> ![Le menu Gameplay avec Portes et mobiles ouvert : porte à charnière avec charnière à gauche ou à droite, porte coulissante vers le haut ou sur le côté, et Ascenseur, Plateforme mobile](../assets/button-door/menu-doors-and-movers.png)

## Laisser le joueur appuyer { #let-the-player-press-it }

L'addon n'a pas de système d'interaction à lui, puisque chaque jeu gère « utiliser » à sa façon. Les portes et les
boutons ont une méthode `use(activator)` et le script de votre joueur l'appelle. Le joueur de démo le fait sur **E** : il
lance un rayon depuis la caméra et appelle `use` sur la première chose touchée qui en a une
([godot/demo/player.gd](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/player.gd)) :

```gdscript
func use_target() -> void:
	var from := camera.global_position
	var query := PhysicsRayQueryParameters3D.create(from, from - camera.global_basis.z * use_distance)
	query.exclude = [get_rid()]
	query.collide_with_areas = false
	var hit := get_world_3d().direct_space_state.intersect_ray(query)
	var node: Node = hit.get("collider")
	while node:
		if node.has_method(&"use"):
			node.call(&"use", self)
			return
		node = node.get_parent()
```

Le joueur se passe lui-même comme activateur, donc `pressed(activator)` emporte le joueur avec lui. Les sorties suivantes
de la chaîne peuvent cibler `!activator` pour agir sur celui qui a appuyé sur le bouton.

## Variantes { #variations }

| Vous voulez | Quoi changer |
| --- | --- |
| Une porte qui ne s'ouvre que depuis le bouton | Décochez `interact` de la porte. Sinon appuyer sur **E** sur la porte l'ouvre aussi |
| Une porte qui se referme toute seule | Réglez `wait` de la porte sur le nombre de secondes où elle reste ouverte. La valeur par défaut, -1, la garde ouverte |
| Une porte verrouillée | Cochez `locked`. La porte ignore `open` et `use` et déclenche `locked_use` à la place, jusqu'à ce qu'une sortie appelle son entrée `unlock`, par exemple depuis un ramassage de clé |
| Un bouton, plusieurs portes | Nommez les portes `gate_1`, `gate_2` et ainsi de suite et ciblez `gate_*`. Un `*` final correspond à tous les noms qui commencent par `gate_` |
| Une porte battante | Utilisez plutôt `func_door_rotating` et faites glisser sa poignée de charnière dans la vue à l'endroit où la charnière doit être |

## Le même câblage en code { #the-same-wiring-in-code }

Vous préférez connecter les choses en GDScript plutôt que dans l'éditeur ? Voici l'équivalent du lien ci-dessus :

```gdscript
# pressed(activator) > main_door.open
button.pressed.connect(func(activator):
	for door in TrenchDotIO.find_targets(button, "main_door", activator):
		TrenchDotIO.invoke(door, &"open", "", activator))
```

Connecter `pressed` directement à `door.open` échoue, parce que `pressed` passe un argument et que `open()` n'en prend
aucun. `invoke` ajuste les arguments à la méthode, comme le font les liens de carte.
