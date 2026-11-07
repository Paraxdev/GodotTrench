---
source: 2bf892f3f08e81fde13de0d18bb8d2c0a06c8753
---

# Decals { #decals }

Les decals posent de la crasse, des taches, des fissures, des affiches et des enseignes sur une surface. Ils sont dessinés
un poil plus près de la caméra que le mur qui est derrière eux et fondus par-dessus, si bien qu'ils ne scintillent jamais
contre le mur ni les uns contre les autres.

## Poser un decal { #placing-a-decal }

1. Trouver le matériau

    Cherchez-le dans le panneau Matériaux, la zone de recherche en haut à gauche aide.

2. Maintenir Alt et le faire glisser sur une face

    La fiche à côté du curseur vous rappelle qu'Alt dépose un decal.

    ![Glisser night/shop_sign sur un mur de briques, la fiche à côté du curseur indique Déposer sur une face, Shift couvre tout le brush, Alt dépose un decal](../assets/decals/decal-alt-drop.png)

3. Relâcher

    Vous devriez voir le decal atterrir là où vous l'avez déposé, sous la forme d'un carré d'un mètre, 32 sur 32 unités, face
    vers l'extérieur de la face.

    ![Le decal de l'enseigne de boutique sur le mur de briques](../assets/decals/decal-on-wall.png)

Un decal est un maillage normal, déplacez-le, tournez-le et redimensionnez-le donc comme tel. Une fois sélectionné,
l'Inspecteur montre **Decal (fondu, double face)** coché, ainsi que les champs de position et de taille pour le placer
exactement.

![L'Inspecteur avec le decal sélectionné, Decal (fondu, double face) coché et sa taille de 32 sur 32](../assets/decals/decal-inspector.png)

Cette même case transforme n'importe quel maillage mince en decal, pour un decal au contour personnalisé.

Les bords doux restent doux. Pour une coupure franche, utilisez un matériau en mode alpha scissor, voir
[Transparence](../godot/materials.md#transparency).

## Pourquoi pas un brush fin { #why-not-a-thin-brush }

> [!WARNING]
> Ne simulez pas des decals avec des brushes fins ou des maillages à quelques millimètres d'un mur. Ils scintillent.

De près, un brush fin a l'air correct. Reculez et il se met à scintiller à travers le mur, en bandes qui rampent quand vous
bougez. C'est ce qu'on appelle le z-fighting.

Le GPU détermine quelle surface est la plus proche à chaque pixel en comparant leurs profondeurs, et il stocke la
profondeur en un nombre limité de pas. Les pas deviennent plus grossiers à mesure qu'une surface est loin et qu'on la
regarde à plat. À quelques mètres, le brush et le mur s'arrondissent au même pas, si bien que chaque pixel choisit l'un
des deux presque au hasard, et le choix change à chaque mouvement de la caméra. Éloigner le brush du mur ne fait que
repousser le scintillement plus loin, et de près il flotte alors visiblement devant le mur.

Un decal n'a pas ce problème. Quand Godot construit la carte, il rapproche chaque decal de la caméra de 0,05 pour cent de
sa distance, un demi-millimètre à 1 mètre et 5 centimètres à 100 mètres, si bien qu'il reste devant le mur à toute
portée. Ce rapprochement se fait le long de votre ligne de visée, donc le decal ne se décale pas à l'écran. Il n'écrit
pas non plus de profondeur propre, si bien que des decals qui se chevauchent se fondent les uns sur les autres au lieu de
se disputer la place.

Un maillage mince convient dès que vous cochez **Decal (fondu, double face)** dessus, ce qui en fait un vrai decal.

## Decals projetés { #projected-decals }

Un decal ordinaire est plat. Pour une tache qui passe par-dessus une marche, un tuyau ou un sol bosselé, utilisez plutôt un
decal projeté. C'est une entité qui projette un matériau sur tout ce qui se trouve dans sa boîte, accessoires compris.
Il coûte plus cher à dessiner, restez donc sur des decals ordinaires pour les surfaces planes.

Il est fourni avec le pack d'entités Gameplay sous le nom `infodecal`. [Decals projetés](projected-decals.md) vous montre
comment le configurer, le placer et à quoi faire attention.

<div class="td-mcp" markdown>

## MCP { #mcp }

Via le [serveur MCP](../mcp.md), placez un decal avec `run_action` `create_decal`, en passant `material`, `at`, `normal`
et `size` sous la forme `[width, height]`.

</div>
