---
source: 2de051fc5fd93ade8349b194b97a6c34a8399111
---

# Decals projetés { #projected-decals }

Un decal projeté est une boîte qui plaque un matériau sur tout ce qu'elle contient. Contrairement à un
[decal ordinaire](decals.md), qui est une feuille plate, il épouse ce qu'il touche, si bien qu'une tache peut déborder
sur une marche, s'enrouler autour d'un tuyau ou reposer sur un sol bosselé. Les props et les modèles dans la boîte le
reçoivent aussi.

C'est un nœud Godot `Decal` dans la scène construite, et il coûte plus cher à dessiner qu'un decal ordinaire. Servez-vous-en
là où une feuille plate ne suffit pas.

> [!WARNING]
> Les decals projetés pèsent plus lourd sur le GPU que les [decals ordinaires](decals.md). Godot calcule le decal pour
> chaque pixel que sa boîte couvre à l'écran, donc beaucoup de boîtes, ou de grandes boîtes, ralentissent le jeu, surtout
> sur les cartes graphiques modestes. Utilisez un decal ordinaire sur une surface plane, gardez la boîte aussi petite que
> la tâche le permet, et réservez les decals projetés aux endroits qui ont besoin de l'enroulement.

## L'ajouter à votre projet { #adding-it-to-your-project }

Le decal projeté vient avec le
[pack d'entités Gameplay](../godot/project-setup.md#core-entities-and-the-gameplay-entities-pack) sous le nom `infodecal`,
donc le mettre en place revient à installer le pack.

1. Installer ou mettre à jour le pack

    Cliquez sur **Installer les entités Gameplay** dans la fenêtre *Configurer ce projet*, ou sur **Mettre à jour les
    entités Gameplay** dans le menu Gameplay quand le pack est déjà dans votre projet.
    [Installer et mettre à jour le pack](../godot/project-setup.md#installing-and-updating-the-pack) vous guide pas à pas.

2. Vérifier que l'éditeur le voit

    Vous devriez voir une carte `infodecal` dans l'onglet **Entités**, à côté de l'Inspecteur. Tapez `infodecal` dans la
    zone de recherche en haut de l'onglet pour la trouver vite.

    ![L'onglet Entités filtré sur infodecal, la carte survolée et son infobulle indiquant Projected decal](../assets/projected-decals/entities-infodecal-card.png)

> [!NOTE]
> Si vous avez créé votre propre définition `infodecal` avant que le pack n'en ait une, retirez-la de votre FGD après la
> mise à jour. Sinon la classe est définie deux fois, et l'éditeur vous en avertit.

> [!TIP]
> Vous n'utilisez pas le pack ? Copiez sa définition,
> [`infodecal.tres`](https://github.com/Paraxdev/trenchdot_func/blob/main/gameplay_pack/definitions/infodecal.tres),
> dans votre projet et ajoutez-la à *Entity Definitions* de votre propre fichier FGD, voir
> [Entités personnalisées](../gameplay/custom-entities.md#making-it-placeable). Gardez *Node Class* sur `Decal`, le script
> en a besoin et c'est ainsi que l'éditeur sait traiter l'entité comme un decal.

## En poser un { #placing-one }

1. Faire glisser la carte sur une surface

    Ouvrez l'onglet **Entités** à côté de l'Inspecteur et faites glisser la carte `infodecal` dans une vue, comme pour
    placer n'importe quelle entité (voir [Placer des entités](../gameplay/placing.md#placing-entities)). Elle se pose là
    où le pointeur rencontre la première surface solide, tournée pour projeter vers l'intérieur de cette surface. Une
    boîte montre l'espace qu'elle couvre, moitié devant la surface et moitié derrière.

    ![Un infodecal au bord d'une marche de briques dans la vue 3D, sa boîte passant par-dessus et le long de la face avant, avec le matériau de l'enseigne prévisualisé à l'intérieur](../assets/projected-decals/decal-box-on-step.png)

2. Choisir le matériau

    Dans l'Inspecteur, tapez un nom de matériau dans `material`, le même nom qu'une face utilise, comme `base/stain`. Sa
    couleur, sa carte normale et sa lueur sont reprises.

    ![L'Inspecteur avec l'infodecal sélectionné, night/shop_sign dans material et 128 64 64 dans size](../assets/projected-decals/decal-inspector-material.png)

3. Régler sa taille

    `size` prend trois nombres en unités de carte : largeur, profondeur et hauteur. La largeur et la hauteur forment la
    zone couverte sur la surface, la profondeur indique jusqu'où la boîte s'enfonce dans la surface et en ressort. La
    valeur par défaut est 64 32 64, un carré de deux mètres qui s'étend d'un demi-mètre de chaque côté. Laissez la valeur
    par défaut et la zone prend plutôt la [taille de texture](../godot/materials.md#texture-size) du matériau, s'il en a
    une, dans l'éditeur comme dans Godot.

4. (Facultatif) L'orienter

    Faites-le tourner comme n'importe quelle autre entité. Il projette toujours le long de son propre axe vers le bas,
    -Y local, vers lequel le dépôt a pointé dans la surface.

5. Construire la carte dans Godot

    Le matériau apparaît sur chaque surface à l'intérieur de la boîte une fois que Godot construit la carte, voir
    [Construire des cartes](../godot/building.md).

    L'éditeur ne montre qu'un aperçu plat du matériau dans la boîte, Godot dessine la vraie projection, en courbant
    l'enseigne par-dessus le bord de la marche et le long de sa face avant.

    ![La même enseigne dans un jeu Godot en cours d'exécution, courbée par-dessus le bord de la marche de briques et sur sa face avant](../assets/projected-decals/decal-in-game.png)

## Clés { #keys }

| Clé | Ce qu'elle fait |
| --- | --- |
| `material` | Le matériau à projeter, un nom comme `base/stain` ou un chemin `res://`. Ses textures d'albédo, de normale, d'ORM et d'émission et sa couleur d'albédo sont utilisées |
| `texture` | Un chemin `res://` vers une image, projetée quand `material` est vide |
| `normal_texture` | Un chemin `res://` vers une carte normale, à utiliser avec `texture`. La carte normale propre à un matériau la remplace |
| `size` | Largeur, profondeur et hauteur de la boîte en unités de carte, 64 32 64 sauf si vous changez la valeur |
| `modulate` | Une couleur par laquelle l'image projetée est multipliée |

## À surveiller { #things-to-watch-out-for }

| Situation | Ce qui se passe |
| --- | --- |
| Le projet utilise le moteur de rendu Compatibility | Godot ne dessine aucun decal. Passez *Rendering Method* sur Forward+ ou Mobile dans *Project Settings > Rendering > Renderer* |
| Le projet utilise le moteur de rendu Mobile | Seuls 8 decals s'affichent sur un même maillage, les autres sont omis |
| Des faces parallèles à la projection | La texture bave en traînées, comme sur la face avant d'une marche sous un decal qui pointe droit vers le bas. Inclinez la boîte pour qu'elle rencontre les deux faces en biais |
| Des surfaces près des extrémités de la boîte | Le decal s'estompe vers le haut et le bas de sa boîte, gardez donc la surface près du milieu, où le dépôt la place |
| Quelque chose traverse la boîte | Cela reçoit aussi le decal, un joueur qui passe y compris. Ne faites pas la boîte plus profonde que la surface ne le demande |
| Verre et découpes | Ils restent aussi transparents qu'avant, un decal ne peut pas combler les parties transparentes |
| Beaucoup de grands decals | Chacun ajoute du travail à chaque pixel qu'il couvre, ils ralentissent donc le jeu là où ils s'accumulent |

<div class="td-mcp" markdown>

## MCP { #mcp }

Via le [serveur MCP](../mcp.md), `run_action` avec `action: "place_entities"` et
`args: {"classnames": ["infodecal"], "at": [x, y, z], "normal": [x, y, z]}` le place tourné vers la surface, comme un
glisser-déposer. Réglez ensuite ses clés avec `update_entity` `properties`.

</div>
