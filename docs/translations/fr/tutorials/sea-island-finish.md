---
source: 5d131fe08a12db6615a97a4cbd455746685c5f02
---

# Île 5 : la jetée et Godot { #sea-island-5-pier-and-godot }

## La jetée { #the-pier }

1. Lui donner son propre calque

    Cliquez sur **Ajouter un calque** en haut de l'Outliner. Le nouveau calque devient le calque courant, donc la jetée
    s'y retrouve et vous pouvez la masquer ou la verrouiller à part du terrain.

    ![L'Outliner avec Ajouter un calque survolé et son infobulle : nouveau calque, il devient le calque courant où vont les nouveaux objets](../assets/sea-island/outliner-add-layer.png)

2. La construire

    Appuyez sur **Q** pour l'outil Sélectionner, car un glissement avec l'outil Scatter peint encore des modèles. Puis
    dessinez de simples brushes qui avancent vers le large depuis le milieu de la plage, dimensionnés comme dans le
    tableau ci-dessous.

3. Ajouter des props

    Allez dans *Fichier > Importer > Prop modèle (.bbmodel, .glb)…* et choisissez des modèles dans le dossier
    `models/polyhaven` du projet de démo : deux `containers/wooden_crate_01` et un `containers/wine_barrel_01` sur le
    pont, un `nautical/lifebuoy` contre un poteau et un `nautical/ocean_buoy` dans l'eau. Chacun atterrit au curseur 3D,
    déplacez-le ensuite à sa place.

    ![Le menu Fichier avec Importer ouvert et Prop modèle (.bbmodel, .glb)… survolé](../assets/sea-island/menu-import-model-prop.png)

| Partie | Taille | Matériau |
| --- | --- | --- |
| Pont | 128 de large, 1100 de long, 8 d'épaisseur, dessus 48 au-dessus de l'eau | `showcase/planks` |
| Poteaux | 16 unités de côté, deux rangées tous les 200 unités, de 320 sous l'eau jusqu'au pont | `showcase/beam` |
| Garde-corps | Deux petits poteaux au bout, 40 de haut | `showcase/beam` |

Un prop référence son modèle par un chemin `res://`, donc le fichier doit se trouver dans le projet Godot ouvert.
Choisissez-en un ailleurs et l'éditeur propose d'abord de le copier dans `res://models`.

![La jetée vue depuis la mer](../assets/sea-island/pier.jpg)

## La voir dans Godot { #see-it-in-godot }

Enregistrez la carte dans votre projet Godot et construisez-la avec un `FuncGodotMap`, voir
[Construire des cartes](../godot/building.md). Chaque partie de la carte devient un autre genre de nœud :

| Partie de la carte | Ce que vous obtenez dans Godot |
| --- | --- |
| Terrain | Un `TrenchDotTerrain` découpé en chunks, chacun un maillage qui mélange les quatre calques et a sa propre collision |
| Mer | Le brush `func_illusionary`, visible sans collision |
| Ensembles d'arbres, de buissons et de blocs rocheux | Des MultiMeshes, plus un corps statique par modèle avec une forme de collision pour chaque instance, toutes partageant une seule forme |
| Ensembles d'herbe | Des MultiMeshes sans collision qui s'estompent avec la distance |

![L'île terminée dans Godot](../assets/sea-island/godot-overview.jpg)

![La jetée dans Godot](../assets/sea-island/godot-pier.jpg)
