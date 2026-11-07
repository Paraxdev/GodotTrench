---
source: be5056a246dacf54d3a4aecafff90bff3f0d1aad
---

# Bibliothèque de prefabs { #prefab-library }

L'onglet **Prefabs** en bas, à côté de Matériaux et de Modèles, contient des pièces toutes faites à coller dans une carte :
escaliers, encadrements de porte, piliers, petites salles et détails. Pensez aux collections de prefabs où copient les
mappeurs TrenchBroom, mais intégrées à l'éditeur.

![L'onglet Prefabs avec Rechercher des prefabs, la liste des catégories, le curseur de taille des cartes et une carte pour chaque pièce intégrée, l'escalier en lacets survolé avec son infobulle : Stairs/Switchback stair, 192 × 128 × 352 unités, Intégré](../assets/prefab-library/prefabs-panel.png)

Chaque carte dessine sa pièce du côté qui en montre le plus : de dessus pour les plans au sol comme un escalier, de face
pour les murs comme un encadrement de porte. Plus c'est clair, plus c'est proche.

## Coller une pièce { #pasting-a-piece }

1. Pointer l'endroit

    Survolez, dans une vue, l'endroit où la pièce doit aller, cela place le curseur 3D.

2. Cliquer sur la carte

    Tapez dans *Rechercher des prefabs* ou choisissez une catégorie pour la trouver, puis cliquez dessus. La pièce est
    collée centrée sur cet endroit et calée sur la grille, comme Ctrl+V.

    ![L'escalier en lacets collé dans une petite salle, ses marches dans les matériaux de dev gris](../assets/prefab-library/pasted-switchback-stair.png)

Seul l'endroit sur le plan au sol suit le curseur. La hauteur reste celle de l'enregistrement de la pièce, donc à un étage
supérieur, montez-la ensuite.

Le collage est une copie simple sans lien avec la bibliothèque, remodelez-la ou retexturez-la donc à volonté. Les pièces
intégrées utilisent les matériaux `dev/` que tout projet possède, retexturez-les depuis l'onglet Matériaux.

Un clic droit sur une carte donne *Coller* et *Copier*. Copier met la pièce dans le presse-papiers, pour la coller dans
une autre carte ou la partager sous forme de texte.

Pour couvrir toute une zone avec des pièces choisies au hasard, utilisez le [Remplissage aléatoire](random-fill.md).

> [!TIP]
> Une pièce collée est du contenu de carte ordinaire. Pour quelque chose qui doit rester synchronisé partout où il sert,
> comme un cadre de fenêtre répété sur tout un bâtiment, faites plutôt une [instance de prefab](organizing.md#prefabs).

## Vos propres pièces { #your-own-pieces }

1. Sélectionner les objets

    Sélectionnez ce qui doit devenir la pièce, avec un projet Godot ouvert.

2. Ajouter la sélection

    Ouvrez **Ajouter la sélection** en haut de l'onglet Prefabs, donnez un nom et une catégorie à la pièce, et cliquez sur
    **Ajouter**.

    ![Ajouter la sélection ouvert, avec Grand stair comme nom, Stairs comme catégorie, Garder la position décoché et l'infobulle d'Ajouter : Enregistre la sélection en .tdm dans res://prefab_library, le centre de son bas sur l'origine](../assets/prefab-library/add-the-selection.png)

3. Trouver la nouvelle carte

    Elle apparaît en premier, avec un cadre bleu. Un clic droit dessus donne *Ouvrir pour modifier*, qui ouvre la pièce
    comme une carte, et *Afficher dans l'explorateur*. Les pièces intégrées ne se modifient pas.

    ![Le menu du clic droit sur la carte Grand stair avec Coller, Copier, Ouvrir pour modifier survolé et Afficher dans l'explorateur](../assets/prefab-library/card-menu.png)

La pièce est enregistrée en `.tdm` dans `res://prefab_library/<category>/`, déplacée pour que le centre du bas de la
sélection se trouve sur l'origine. Cochez **Garder la position** pour la laisser où elle est, pour une pièce que vous avez
construite exprès autour de l'origine de la carte, comme les pièces d'un kit de [remplissage aléatoire](random-fill.md).

Chaque `.tdm` de ce dossier est une pièce. Enregistrez-y des cartes vous-même avec *Fichier > Enregistrer sous…* et elles
apparaissent aussitôt comme des cartes. Pour des fichiers copiés à la main, ou un dossier de pièces partagé entre projets,
appuyez sur le bouton ⟳.

## Coller depuis TrenchBroom { #pasting-from-trenchbroom }

TrenchDot colle aussi ce que TrenchBroom met dans le presse-papiers. Copiez des brushes dans TrenchBroom, ou depuis une
collection de prefabs TrenchBroom sur le web, et appuyez sur Ctrl+V dans une carte. Les groupes arrivent comme des
groupes, les entités comme des entités, et les axes sont tournés comme lors de l'[import d'un .map](importing.md). Les noms
de texture sont conservés, les textures d'outil d'autres jeux deviennent celles du projet.

<div class="td-mcp" markdown>

## MCP { #mcp }

L'outil `prefab_library` liste les pièces (`op: list`), en colle une par son `category/name` (`op: paste`, avec `origin`
ou `offset`), renvoie son texte de presse-papiers (`op: copy`), et ajoute des objets à la bibliothèque du projet (`op: add`,
la sélection sauf si des `ids` sont donnés). `run_action` avec `paste` accepte le texte de presse-papiers de TrenchBroom
comme celui de TrenchDot.

</div>
