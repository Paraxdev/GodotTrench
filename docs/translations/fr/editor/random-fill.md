---
source: 7bb83afeb1e5ebd8423b507ac79851c690337899
---

# Remplissage aléatoire { #random-fill }

*Brush > Remplissage aléatoire…* couvre une grille de cellules avec des pièces de la
[bibliothèque de prefabs](prefab-library.md), en choisissant la pièce de chaque cellule selon son poids. Construisez un
petit kit de pièces, comme une salle de bureau, un couloir, une grille de piliers et un sol dégagé, et le remplissage
dessine tout un plan d'étage à partir d'elles. Chaque graine donne un plan différent, et la même graine donne toujours le
même.

1. Construire vos pièces

    Construisez chaque pièce d'une cellule de large (128 unités par défaut), centrée sur l'origine. Gardez les murs à
    l'écart de la bordure de la cellule, pour que des pièces voisines se raccordent toujours.

    Ajoutez-la ensuite à la bibliothèque : sélectionnez-la, ouvrez **Ajouter la sélection** en haut de l'onglet Prefabs,
    cochez **Garder la position**, donnez-lui un nom et cliquez sur **Ajouter** (voir
    [Vos propres pièces](prefab-library.md#your-own-pieces)).

    ![L'onglet Prefabs avec Ajouter la sélection ouvert et Garder la position coché, son infobulle disant qu'il garde l'origine de la carte comme origine de la pièce, comme un kit pour le Remplissage aléatoire](../assets/random-fill/prefabs-keep-position.png)

2. Ouvrir la boîte de dialogue

    Allez dans *Brush > Remplissage aléatoire…*.

    ![Le menu Brush avec Remplissage aléatoire… survolé](../assets/random-fill/menu-random-fill.png)

3. Ajouter les pièces

    Choisissez chaque pièce dans **Ajouter une pièce…** et donnez-lui un **poids** : une pièce de poids 6 remplit environ
    deux fois plus de cellules qu'une de poids 3. **tourner** permet à une pièce de tourner d'un multiple de 90 degrés
    choisi au hasard.

    ![La boîte de dialogue Remplissage aléatoire avec trois pièces, leurs poids, poids vide 4, une grille de 12 sur 8 et son aperçu en couleurs](../assets/random-fill/random-fill-dialog.png)

4. (Facultatif) Laisser des cellules ouvertes

    **poids vide** fonctionne comme une pièce qui ne place rien.

5. Placer la grille

    Sélectionnez le sol et cliquez sur **Ajuster à la sélection**, ou réglez vous-même la taille de **Cellule**, le nombre de
    **Cellules** et le **Coin**.

6. (Facultatif) Regrouper les pièces

    **Regroupement** rassemble chaque pièce en zones d'environ **Taille des zones** cellules, si bien qu'une aile de bureaux
    jouxte une salle de piliers au lieu que tout soit mélangé uniformément.

7. Remplir

    La grille colorée montre un aperçu de la disposition, une couleur par pièce. Cliquez sur **Remplir**, et **Retirer au
    sort** remplace le dernier remplissage par la graine suivante.

    ![Un sol rempli de piliers carrés et ronds et de piles de caisses, avec quelques cellules laissées vides](../assets/random-fill/random-fill-result.jpg)

Le remplissage est un seul groupe de copies simples, il se précalcule et se construit donc comme tout ce que vous placez à
la main, et vous pouvez modifier n'importe quelle cellule ensuite. Les noms d'entité de chaque copie reçoivent leur propre
préfixe, `rf<seed>_<n>-`, si bien qu'une pièce avec des entités reliées, comme une lampe qui clignote sur une minuterie,
fonctionne seule dans chaque cellule.

> [!TIP]
> Remplissez différents types de détails en passes séparées sur la même grille, par exemple un remplissage pour les murs
> et un pour les lumières du plafond, avec leurs propres poids, comme quelques lumières mortes ou qui clignotent parmi
> celles qui fonctionnent.

<div class="td-mcp" markdown>

## MCP { #mcp }

L'outil `random_fill` prend `pieces` sous la forme `[{name, weight, rotate}]`, `empty`, `min`, `cell`, `cells`, `seed`,
`cluster` et `cluster_size`. `preview: true` renvoie seulement la disposition sous forme d'esquisse, `replace` retire
d'abord un remplissage précédent.

</div>
