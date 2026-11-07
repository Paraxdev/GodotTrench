---
source: 5b022e12202fe97edce0105dd2916cbd30e66e36
---

# Terrain { #terrain }

Un terrain est une grande grille de hauteurs que vous sculptez comme de l'argile, avec jusqu'à quatre textures peintes
dessus. Servez-vous-en pour le sol à ciel ouvert, comme des collines, des plages et des vallées, là où façonner des brushes
à la main prendrait une éternité.

Cette page explique comment en créer un et le modeler avec l'outil Sculpter (G). Le texturer est expliqué dans
[Calques de terrain](terrain-layers.md) et [Peinture de terrain](terrain-painting.md).

## Créer un terrain { #creating-a-terrain }

1. Ouvrir la boîte de dialogue

    Allez dans *Terrain > Créer un terrain…* dans la barre de menus.

    ![Le menu Terrain ouvert avec Créer un terrain… survolé](../assets/terrain/menu-create-terrain.png)

2. Choisir une taille

    **Résolution** et **Taille de cellule** décident de sa taille, et la ligne **Taille** en dessous montre le résultat en
    unités et en mètres. La valeur par défaut de 129 x 129 sommets avec des cellules de 64 unités donne 8192 unités, soit
    256 m. Voir [Choisir la taille de cellule](#picking-the-cell-size) plus bas.

    ![La boîte de dialogue Créer un terrain avec ses valeurs par défaut : 129 x 129, cellules de 64 u, Collines, et les quatre champs de calque](../assets/terrain/create-terrain-dialog.png)

3. (Facultatif) Choisir une forme de départ

    **Forme** est le sol dont vous partez. C'est Collines par défaut, les autres sont Plat, Montagne, Île, Vallée et
    Crêtes. **Importer un PNG…** lit à la place une carte de hauteur en niveaux de gris, où les pixels plus clairs sont plus
    hauts. Les deux ne marchent qu'ici, à la création.

4. (Facultatif) Choisir les calques

    **Calque de base**, **Calque de pente**, **Calque de sommet** et **Calque bas** sont les quatre textures du terrain, et
    **Utiliser l'actuel** met le matériau sur lequel vous avez cliqué en dernier. Avec **Peindre les calques selon pente et
    hauteur** coché, ils sont peints tout de suite, voir [Peinture auto](terrain-painting.md#auto-paint).

5. Cliquer sur **Créer** en bas

    Le terrain est centré sur le curseur 3D, ou sur le **Centre** que vous saisissez avec **Curseur 3D** décoché, et cette
    hauteur devient le zéro du terrain. Vous devriez voir le nouveau terrain sélectionné, avec l'outil Sculpter déjà actif.

## Choisir la taille de cellule { #picking-the-cell-size }

| Réglage | Signification |
| --- | --- |
| Résolution | Combien de sommets courent le long de chaque côté, de 33 à 1025. Plus de sommets retiennent plus de détail mais coûtent plus cher à dessiner et à sculpter |
| Taille de cellule | La distance en unités de carte entre deux sommets voisins |

257 sommets avec des cellules de 64 unités, par exemple, donnent 16384 unités, soit 512 m.

Rien ne se trouve entre deux sommets, la taille de cellule est donc le plus fin détail que le terrain peut contenir. Une
chaîne de montagnes veut de grandes cellules, un jardin de petites.

Mauvais choix ? Sélectionnez le terrain et ouvrez **Rééchantillonner** dans l'Inspecteur. Il reconstruit la grille à 65,
129, 257 ou 513 sommets le long du plus grand côté et garde les hauteurs, la peinture et les trous. Le terrain garde sa
taille, donc plus de sommets veut dire des cellules plus petites.

![L'Inspecteur du terrain avec le menu Rééchantillonner ouvert et 257 sommets survolé](../assets/terrain/inspector-resample.png)

**Cellules par chunk** (32 par défaut) découpe le terrain en morceaux carrés pour le dessin et la collision. De petits
chunks se reconstruisent plus vite pendant le sculptage, de plus grands donnent moins de nœuds dans Godot.

## Sculpter { #sculpting }

Appuyez sur G, choisissez un mode dans la barre d'options de l'outil en haut à gauche, et faites glisser sur le terrain.
Cela agit sur les terrains sélectionnés, ou sur tous quand rien n'est sélectionné.

![Le menu des modes de l'outil Sculpter ouvert dans la barre d'options de l'outil, à côté des champs de rayon et de force](../assets/terrain/sculpt-modes.png)

Gardez le bouton de la souris enfoncé et le sol continue de monter sous le brush.

![Maintenir Élever avec un brush de 900 unités fait sortir une colline de l'île](../assets/terrain/sculpt-raise.gif)

| Mode | Ce qu'il fait |
| --- | --- |
| Élever, Abaisser | Monte la hauteur ou la creuse tant que vous maintenez le bouton. Un clic fait une petite bosse |
| Lisser | Détend chaque sommet vers ses voisins, pour adoucir les bosses et les crêtes vives |
| Aplanir | Tire le sol à la hauteur où le trait a commencé, pour construire des plateformes, des routes et des plateaux |
| Bruit | Ajoute de petites bosses aléatoires, pour qu'un sol plat paraisse moins artificiel |
| Terrasser | Cale les hauteurs sur des paliers de **pas** unités, pour des rizières ou des falaises en gradins. Le champ apparaît à côté du mode |
| Peindre l'alpha, Effacer l'alpha, Peindre le calque | Peignent au lieu de façonner. Les modes alpha sont prévus pour les displacements, Peindre le calque peint le calque de terrain réglé à côté du mode. Sur un terrain, l'[outil Mélange](terrain-painting.md) le fait avec plus de contrôle |
| Trou, Reboucher | Découpe des cellules entières ou les referme, collision comprise. Utile pour une entrée de grotte ou de bunker |

Ctrl+molette redimensionne le brush, Shift échange élever et abaisser, et maintenir Ctrl lisse quel que soit le mode.

**Force** règle la vitesse à laquelle le brush travaille. Élever et Abaisser déplacent le centre du brush de 8 unités par
seconde par point de force, quel que soit le rayon, si bien que la valeur par défaut de 4 construit 32 unités par seconde.
Pour des collines et des îles, montez la force plutôt que de maintenir pendant une minute.

Gardez le rayon au-dessus de la taille de cellule du terrain, un brush plus petit ne pousse que le seul sommet qu'il a
dessous (la barre d'état vous prévient quand cela arrive).

Pour recommencer, *Terrain > Sculpter > Aplanir*, ou **Aplanir** dans l'Inspecteur, remet tout le terrain sélectionné à
sa hauteur zéro.

> [!TIP]
> Modelez avec un grand brush et lissez après chaque passe grossière. Les petits brushes laissent des bosses.

## Dans Godot { #in-godot }

Chaque terrain devient un nœud `TrenchDotTerrain`, un `StaticBody3D` (le nœud Godot pour un sol solide qui ne bouge
jamais) dont la collision correspond à ce qu'il dessine. Pour le code de gameplay, `height_at(x, z)` renvoie la hauteur
du sol à une position locale, pratique pour poser des choses sur le sol à l'exécution.

<div class="td-mcp" markdown>

## MCP { #mcp }

L'opération `sculpt` de l'outil `terrain_edit` applique un seul coup de pinceau, et sa force est la distance dont ce coup
déplace le centre, là encore quel que soit le rayon. Maintenir un trait pendant une seconde à la force 4 équivaut à un
coup de force 32.

</div>
