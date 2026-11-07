---
source: be4d0912557053b84612829a7948190773c48b4d
---

# Île 1 : le terrain { #sea-island-1-terrain }

Une grande île avec deux collines, des falaises, une plage, des chemins, un bois et une jetée, construite en cinq
parties. Les nombres sont ceux utilisés pour les images, en unités de carte à 32 par mètre.

![L'île terminée vue du sud](../assets/sea-island/island-finished.jpg)

## Créer le terrain { #create-the-terrain }

Allez dans *Terrain > Créer un terrain…* dans la barre de menus.

![Le menu Terrain avec Créer un terrain… survolé](../assets/sea-island/menu-create-terrain.png)

Remplissez la boîte de dialogue ainsi :

| Réglage | Valeur et pourquoi |
| --- | --- |
| Résolution, Taille de cellule | 257 x 257 points, espacés de 64 unités. La ligne Taille en dessous montre ce que ça donne, 16384 unités (512 m) |
| Forme | Île, Hauteur 1600, Taille des reliefs 3000, Érosion 20, Graine 1. Île monte au milieu et retombe vers les bords. La taille des reliefs est la largeur des plus grosses bosses, et l'érosion laisse les pentes raides s'affaisser pour qu'elles paraissent usées par le temps |
| Calque de base, de pente, de sommet, bas | `showcase/grass`, `showcase/cliff`, `showcase/dirt`, `showcase/sand`. Tuile 256 pour les deux premiers, 192 pour les autres. La base couvre tout ce que vous ne peignez pas, les autres sont faits pour le terrain raide, haut et bas |
| Centre | Décochez **Curseur 3D**, puis `0 -576 0` |
| Peindre les calques selon pente et hauteur | Décoché, la partie 3 peint à la main |

(La valeur de tuile est le nombre d'unités que couvre une répétition de la texture.)

![La boîte Créer un terrain remplie : 257 x 257, taille de cellule 64, Île, hauteur 1600, taille des reliefs 3000, érosion 20, graine 1, les quatre calques showcase, centre 0 -576 0 et Peindre les calques selon pente et hauteur décoché](../assets/sea-island/create-terrain-dialog.png)

Cliquez sur **Créer** en bas. Vous devriez voir apparaître une île verte, déjà sélectionnée, et l'éditeur passe tout seul
à l'outil Sculpter.

La mer sera à la hauteur 0, donc abaisser le terrain de 576 inonde son anneau extérieur et laisse le milieu au sec. Les
parties suivantes désignent les calques par leur numéro d'emplacement : 0 herbe, 1 falaise, 2 terre, 3 sable.

## Ajouter la mer { #add-the-sea }

La mer est un seul brush plat dans lequel le joueur peut marcher.

1. Dessiner une énorme boîte

    Appuyez sur **Q** pour l'outil Sélectionner et sur **Échap** pour que le terrain ne soit plus sélectionné, sinon votre
    glissement le sculpte ou le déplace. Puis faites glisser dans la vue de dessus pour tracer une boîte bien plus grande
    que le terrain, sinon son bord se voit à l'horizon.

2. L'aplatir

    Dans la vue de face, donnez-lui 32 unités d'épaisseur avec son dessus à la hauteur 0.

3. En faire de l'eau

    Boîte sélectionnée, cliquez sur `showcase/water` dans l'onglet Matériaux. Puis Shift+clic sur l'eau dans la vue 3D pour
    ne sélectionner que sa face du dessus, et réglez *échelle* sur 8 et 8 dans la section *Alignement* de l'Inspecteur.
    Ainsi la texture de l'eau ne se répète pas tous les quelques mètres.

    ![L'Inspecteur avec 1 face sélectionnée, matériau showcase/water et l'échelle de l'Alignement à 8,00 et 8,00](../assets/sea-island/inspector-alignment-scale.png)

4. Laisser le joueur y entrer

    Cliquez une fois sur l'eau, loin de l'île, pour que toute la boîte soit de nouveau sélectionnée et plus seulement sa
    face du dessus. Puis faites un clic droit dessus et choisissez *Créer une entité brush > func_illusionary*. Elle
    s'affiche sans collision, donc le joueur peut entrer dans l'eau.

Vous devriez maintenant avoir une île dans la mer, prête à être sculptée :

![L'île générée dans la mer, avant le sculptage](../assets/sea-island/fresh-island.jpg)

Suite : [Île 2 : sculpter](sea-island-sculpt.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

Le [serveur MCP](../mcp.md) laisse des agents et des scripts piloter l'éditeur.
[examples/mcp/sea_island.json](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/sea_island.json) est un
script MCP qui construit la même île, vous pouvez donc le rejouer pour vérifier une valeur.

</div>
