---
source: e0934aeaf4aaffb804a36e5ff5821a38614fc413
---

# Importer des cartes Hammer et TrenchBroom { #importing-hammer-and-trenchbroom-maps }

Vous avez un niveau fait avec Hammer ou TrenchBroom ? Importez-le au lieu de le reconstruire. *Fichier > Importer* ouvre un
`.vmf` de Hammer, ou un `.map` de TrenchBroom, de Quake ou de Qodot, comme une nouvelle carte non enregistrée (dans un
nouvel onglet si la carte ouverte a des modifications non enregistrées). Enregistrez-la en `.tdm` et Godot la construit
comme n'importe quelle autre carte.

## Étapes { #steps }

1. Ouvrez d'abord le projet Godot

    Ouvrez le projet auquel la carte est destinée avec *Godot > Ouvrir un projet Godot…*, puisque les textures converties
    vont dans son dossier de textures.

2. Importez la carte

    Allez dans *Fichier > Importer* et choisissez *Fichier .vmf (Hammer)…* ou *Fichier .map (TrenchBroom, Quake)…*, puis
    le fichier.

    ![Le menu Fichier avec Importer ouvert et Fichier .vmf (Hammer)… survolé, en dessous Convertir les textures et les imports de modèles](../assets/importing/file-import-menu.png)

3. Convertissez les textures

    Quand des textures de la carte se trouvent près d'elle, l'éditeur demande « Convertir les textures de la carte ? »
    pour celles qui manquent au projet. Répondez oui, voir [Convertir des textures](importing-textures.md).

4. Regardez l'onglet Problèmes

    Ouvrez l'onglet Problèmes en haut à gauche, à côté d'Outliner et de Historique. Chaque matériau qui manque encore au
    projet apparaît comme un avertissement, et ses faces s'affichent en damier.

    ![L'onglet Problèmes avec deux avertissements, Le matériau 'castle/brick_wall' n'est pas dans le projet (12 faces), et la vue 3D avec ces murs et ces marches dessinés en damier gris](../assets/importing/issues-missing-materials.png)

    Les faces gardent le nom du matériau, donc ajouter la texture plus tard les corrige. Une fois que toutes les textures
    sont là, ces avertissements disparaissent.

    ![La même carte après la conversion de ses textures : l'onglet Problèmes indique Aucun problème trouvé et les murs montrent leur texture de briques](../assets/importing/textures-converted.png)

5. Enregistrez-la

    Utilisez *Fichier > Enregistrer sous…* et enregistrez-la en `.tdm` dans le projet.

> [!NOTE]
> Les unités sont conservées telles quelles. Les cartes Source sont construites pour environ 40 unités par mètre et les
> cartes Quake pour environ 32, donc une carte Source ressort un peu plus grande que dans son jeu, sauf si le projet
> utilise une autre échelle.

## Ce que l'import conserve { #what-the-import-keeps }

| Hammer | Dans TrenchDot |
| --- | --- |
| Matériaux d'outil (`tools/toolsnodraw`, `toolsclip`, `toolsplayerclip`, `toolstrigger`, `toolsorigin`, ...) | Les textures skip, clip et origin du projet et `special/trigger` |
| Faces de ciel | La texture de ciel du projet, `special/sky` par défaut |
| Displacements | Des displacements avec leurs hauteurs et leurs alphas de mélange |
| `func_instance` | Un groupe qui contient le contenu de l'instance, déplacé à sa place. Les noms à l'intérieur reçoivent le fixup de l'instance, ses remplacements `$` et `#` sont remplis, et les sorties dirigées vers `instance:name;Input` sont câblées vers la vraie entité |
| Visgroups | Calques |
| Objets masqués | Nœuds masqués |
| Clés et sorties d'entité | Propriétés et sorties d'entité |

Les calques et les groupes de TrenchBroom restent des calques et des groupes, et les textures d'outil de Quake deviennent
de la même façon les textures d'outil du projet.

Vous n'avez besoin que de quelques brushes ? Copiez-les dans TrenchBroom et appuyez sur Ctrl+V dans TrenchDot, ils arrivent
convertis de la même façon, voir [Bibliothèque de prefabs](prefab-library.md#pasting-from-trenchbroom).

Les faces de ciel entrent en collision dans Godot mais ne dessinent rien. La clé `sky_source` de worldspawn garde la
texture qu'elles avaient, et convertir les textures la transforme en ciel procédural, voir
[Convertir des textures](importing-textures.md#skies).

## Limites connues { #known-limitations }

| Limite | Que faire |
| --- | --- |
| Les textures dans des archives `.vpk` ou `.pak` ne sont pas lues | Extrayez-les d'abord, par exemple avec VPKEdit |
| Les instances sont cherchées à côté de la carte et dans les dossiers au-dessus | Gardez l'organisation du dossier `maps` du jeu. Les instances introuvables restent des entités `func_instance` |
| Les classes d'entité de Valve et de Quake n'ont pas de définition dans un nouveau projet, donc l'éditeur ne connaît pas leurs clés | Ajoutez-les à votre FGD, ou remplacez-les par vos propres classes. L'onglet Problèmes les liste |
| Certaines fonctions des matériaux Source sont ignorées : mélanges `$basetexture2`, textures de détail, env maps et proxies | Réglez-les dans Godot |
| Les patchs Quake 3 (surfaces courbes) et les primitives de brush sont ignorés. La barre d'état dit combien | Reconstruisez-les en maillages |
| Les displacements dont les sommets bougent sur le côté, pas seulement de haut en bas, deviennent des maillages sans alphas de mélange | Modifiez-les comme des maillages |
