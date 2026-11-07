---
source: cb5ad8fa1eb282094d0cdec538d72382b939f070
---

# Précalculer l'éclairage { #baking-lighting }

Le précalcul calcule une fois pour toutes la lumière du soleil, du ciel, de vos lampes et des matériaux lumineux, rebonds
compris, et la stocke dans la carte sous forme de lightmap. Godot dessine alors cette lumière au lieu de la calculer à
chaque image. Vous obtenez une lumière de rebond douce, des débordements colorés des lampes et des coins plus sombres pour
presque rien à l'exécution. Les lumières que vous voulez allumer, éteindre ou déplacer restent en temps réel à côté du
précalcul.

![À gauche : une salle en Aperçu éclairé, grise et plate avec des taches de soleil aux bords durs sous les fenêtres. À droite : la même salle après le précalcul, lumière chaude des lampes, coins plus sombres et taches de soleil douces](../assets/light-baking/lit-preview-and-baked.png)

Le précalcul couvre la géométrie statique : brushes du monde, maillages, terrains, displacements et entités brush qui ne
bougent pas. Les entités brush mobiles comme les portes et les plateformes, les volumes déclencheurs, les decals, les
textures d'outil et les calques laissés hors de l'export ne sont ni éclairés par le précalcul ni ne projettent d'ombres
précalculées.

## Précalculer une carte { #baking-a-map }

1. Ouvrir la fenêtre Précalculer l'éclairage

    Choisissez *Godot > Précalculer l'éclairage…*.

    ![Le menu Godot avec Précalculer l'éclairage survolé et son infobulle affichée](../assets/light-baking/menu-bake-lighting.png)

2. Vérifier ce qui entre

    La ligne du haut résume le tout, par exemple « Précalcule le soleil, 1 lumière, le ciel et les matériaux lumineux dans
    102 surfaces », et indique quelles lumières restent en temps réel.

    ![La fenêtre Précalculer l'éclairage avec Qualité réglée sur Moyen, Taille de texel 16.0 u, Douceur des ombres 0.30 et S'exécute sur CPU](../assets/light-baking/bake-dialog.png)

3. Choisir une qualité et une taille de texel

    Voir [les options](#dialog-options) plus bas. Les valeurs par défaut conviennent pour un premier coup d'œil.

4. Cliquer sur **Précalculer**

    Une fenêtre de progression montre l'étape, et vous pouvez continuer à éditer entre-temps. Les changements faits pendant
    le précalcul n'y sont pas. **Annuler** l'arrête et la carte garde l'éclairage qu'elle avait.

5. Regarder le résultat

    Une fois terminé, la vue 3D passe à l'ombrage **Éclairage précalculé**, et la barre d'état indique le temps pris et la
    taille de la lightmap.

Le précalcul est enregistré dans le `.tdm`, et précalculer est une seule étape d'annulation. **Supprimer le précalcul**
dans la même fenêtre le retire, Godot éclaire alors de nouveau la carte en temps réel.

## Options de la fenêtre { #dialog-options }

| Option | Ce qu'elle fait |
| --- | --- |
| Qualité | Combien de rayons et de rebonds le traceur dépense, voir le tableau plus bas |
| Taille de texel | Unités de carte couvertes par un texel de lightmap, de 2 à 128, 16 (50 cm) par défaut. Plus petit est plus net et plus lent |
| Douceur des ombres | De 0 à 1. Étale les lumières qui n'ont pas de `light_size` propre pour que les ombres aient des bords doux. 0 les garde dures |
| S'exécute sur | CPU, GPU ou CPU + GPU, voir plus bas |

| Qualité | Rayons par texel | Rebonds | Rayons d'ombre par lumière | À utiliser pour |
| --- | --- | --- | --- | --- |
| Aperçu | 32 | 1 | 4 | Placer des lumières, bruité mais rapide |
| Moyen | 96 | 2 | 8 | Le travail de tous les jours, la valeur par défaut |
| Élevé | 192 | 3 | 12 | Un résultat propre |
| Final | 384 | 4 | 16 | Le build que vous livrez, lent sur les grandes cartes |

La lightmap fait au plus 4096 texels de côté. Quand une carte ne tient pas à la taille de texel demandée, le précalcul en
utilise une plus grossière et le dit dans la barre d'état. La carte se souvient des options avec lesquelles elle a été
précalculée en dernier.

### CPU, GPU ou les deux { #cpu-gpu-or-both }

| S'exécute sur | À choisir quand |
| --- | --- |
| CPU | Fonctionne toujours, sur tous les cœurs. La valeur par défaut sûre |
| GPU | La machine a une carte graphique dédiée, ce qui est en général bien plus rapide |
| CPU + GPU | Les deux sont puissants. La carte et chaque cœur prennent du travail dans une même file, chacun autant qu'il peut en suivre |

Quand le GPU ne peut pas servir, par exemple parce qu'il n'y en a pas, que c'est un rendu logiciel ou que la carte est
trop grande pour ses tampons, le CPU précalcule la carte et la barre d'état dit pourquoi. Un GPU qui échoue en cours de
route passe le reste au CPU.

## Regarder le précalcul { #looking-at-the-bake }

*Affichage > Ombrage > Éclairage précalculé* montre le précalcul, et une fois que la carte en a un, F4 l'inclut quand il
fait défiler l'ombrage. *Affichage > Ombrage > Vue précalculée* choisit ce qu'il montre.

![Affichage > Ombrage > Vue précalculée ouvert, avec Textures avec lumière précalculée coché et Lightmap seule survolé](../assets/light-baking/menu-baked-view.png)

| Vue précalculée | Montre |
| --- | --- |
| Textures avec lumière précalculée | Les textures éclairées par la lightmap, proche de ce que montre Godot |
| Lightmap seule | La lumière seule, sans textures, pour juger le rebond et le bruit |
| Masque d'ombre du soleil | Quelle part du soleil atteint chaque endroit, blanc en plein soleil |
| Occlusion ambiante | L'assombrissement dans les coins et sous les objets, pour vérifier seulement, Godot ne s'en sert pas |

![La même salle dans les quatre vues précalculées : textures avec lumière précalculée en haut à gauche, la lightmap seule en haut à droite, le masque d'ombre du soleil en bas à gauche et l'occlusion ambiante en bas à droite](../assets/light-baking/baked-views.png)

## Clés de lumière { #light-keys }

Chaque `light` et `light_spot` a une clé `bake_mode`, et worldspawn a `sun_bake_mode` pour le soleil :

| Valeur | Dans le précalcul | Dans Godot |
| --- | --- | --- |
| `auto` | Comme `baked`, mais une lumière avec un `targetname` ou `start_on` à 0 reste en temps réel, puisque les E/S peuvent l'allumer ou l'éteindre. Le soleil est toujours précalculé | Statique ou désactivée |
| `baked` | Lumière directe et rebondie et ses ombres | Statique, la lightmap la porte |
| `bounce` | Seulement la lumière rebondie | Dynamique, Godot dessine en temps réel la lumière directe et ses ombres |
| `realtime` | Laissée de côté | Désactivée, entièrement en temps réel |

Utilisez `bounce` pour une lampe dont les ombres doivent suivre les objets mobiles alors que son rebond remplit toujours la
salle. Les noms propres à Godot `static`, `dynamic` et `disabled` fonctionnent aussi. Une lumière en temps réel éclaire
aussi les objets mobiles grâce aux [sondes de lumière](../godot/baked-lighting.md#light-probes).

`light_size` est le rayon de la lampe en mètres. Une lampe plus grosse projette des ombres plus douces, et 0 utilise à la
place la douceur des ombres de la fenêtre. `light_indirect_energy` met à l'échelle la lumière rebondie, et
`omni_attenuation`, `spot_attenuation` et `spot_angle_attenuation` façonnent l'atténuation. La lumière Godot les reçoit
toutes comme propriétés du même nom, si bien que la lumière en temps réel correspond au précalcul.

## Quand un précalcul devient périmé { #when-a-bake-goes-out-of-date }

Le précalcul se souvient de la géométrie et des lumières dont il est issu. Déplacer, remodeler, ajouter ou supprimer de la
géométrie statique, ou changer une lumière, une sonde de lumière, les clés de soleil et de ciel de worldspawn, les
textures utilisées ou leurs réglages, le rend périmé. La fenêtre Précalculer l'éclairage avertit alors que la carte a
changé depuis le dernier précalcul.

![La fenêtre Précalculer l'éclairage avec la ligne jaune « La carte a changé depuis le dernier précalcul. Relancez-le pour mettre à jour l'éclairage. » et les boutons Précalculer et Supprimer le précalcul](../assets/light-baking/bake-dialog-out-of-date.png)

Tant que vous ne précalculez pas de nouveau, les faces qui ont changé montrent l'Aperçu éclairé dans l'éditeur et la
lumière moyenne de la carte dans Godot, tandis que le reste garde sa lumière précalculée. Ici le pilier a été déplacé après
le précalcul : il montre l'Aperçu éclairé, et son ancienne ombre est toujours sur le sol. Précalculez de nouveau avant de
livrer.

![La salle précalculée après le déplacement du pilier, le pilier est éclairé uniformément et son ancienne ombre précalculée reste là où il se tenait](../assets/light-baking/bake-out-of-date.png)

## Réglages de texture { #texture-settings }

Sélectionnez une face et ouvrez **Réglages de texture** dans l'Inspecteur pour régler le traitement de chaque face qui a
cette texture, dans toute la carte. Les réglages sont enregistrés avec la carte.

![L'Inspecteur avec une face sélectionnée et Réglages de texture ouvert : Précalculer la lumière dessus, Projette des ombres précalculées, Détail de lightmap, Répéter tous les et Projection](../assets/light-baking/texture-settings.png)

| Réglage | Ce qu'il fait | À utiliser pour |
| --- | --- | --- |
| Précalculer la lumière dessus | Désactivé, les faces restent hors de la lightmap, Godot leur donne la lumière moyenne de la carte | Ciel, eau, faces que personne ne voit de près |
| Projette des ombres précalculées | Désactivé, la lumière traverse les faces dans le précalcul | Verre, grilles, decals minces |
| Détail de lightmap | Multiplie la taille de texel sur ces faces, en dessous de 1 c'est plus net, au-dessus de 1 cela économise de la place | Des ombres nettes sur un sol, floues sur un plafond lointain |
| Répéter tous les | Unités de carte couvertes par une répétition de la texture, au lieu de sa taille en pixels | Une seule échelle pour une texture partout |
| Projection | Monde ignore le décalage, l'échelle et la rotation propres à chaque face, si bien que la texture se prolonge sans raccord | Papier peint, carrelage, béton |

> [!TIP]
> Pour une carte façon backrooms, réglez le papier peint sur la projection Monde et une taille de répétition une fois pour
> toutes, et chaque mur que vous dessinez avec s'aligne sans toucher à l'outil Texture.

Une face dont les deux options de précalcul sont désactivées est entièrement laissée hors du précalcul. La taille de
répétition et la projection s'appliquent aussi dans Godot.

<div class="td-mcp" markdown>

## MCP { #mcp }

L'outil `bake_lighting` précalcule avec les mêmes options que la fenêtre, `quality`, `texel_size`, `softness` et
`backend` (`cpu`, `gpu` ou `hybrid`), et `view` change la Vue précalculée. Il attend la fin du précalcul et renvoie la
taille de la lightmap, les comptes, le GPU utilisé ou pourquoi il ne l'a pas été, et `out_of_date`. L'opération `settings`
de l'outil `texture` lit et règle les réglages de texture, avec `material`, `bake`, `casts`, `texel_scale`, `size`
(`null` l'efface) et `projection` (`face` ou `world`). Voir [Serveur MCP](../mcp.md).

</div>
