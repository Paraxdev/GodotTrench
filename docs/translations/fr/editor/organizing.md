---
source: b549cb4d1d3fd2ab8e0ac144848e50eac2b168fc
---

# Organiser une carte { #organizing-a-map }

Quand une carte grossit, vous voulez la ranger en parties, retrouver des choses et garder au même endroit les pièces que
vous réutilisez. Presque tout commence dans l'Outliner, le panneau de gauche.

## Calques { #layers }

L'Outliner montre la carte comme un arbre de calques, de groupes et d'objets. Les calques vous permettent de découper une
carte en parties, par exemple un bâtiment par calque, pour masquer ou verrouiller une partie pendant que vous travaillez
sur le reste.

Cliquez sur **Ajouter un calque** en haut de l'Outliner pour en créer un, il devient tout de suite le calque actuel. Les
nouveaux objets vont dans le calque actuel, celui marqué *(actuel)*, et un clic sur un calque le rend actuel. L'œil devant
une ligne la masque, le cadenas l'empêche d'être sélectionnée ou modifiée.

Survolez une ligne pour voir ses matériaux, sa taille et ses limites. L'objet est entouré dans toutes les vues en même
temps, donc vous voyez de quel mur il s'agit pour `brush7`.

![L'Outliner avec le calque House ouvert et la ligne brush7 survolée, une infobulle montre son matériau, sa taille et ses limites pendant que les vues 3D et de face entourent le mur du fond](../assets/organizing/outliner-hover.png)

Les brushes et les maillages sont listés par type et par identifiant, comme `brush12` ou `mesh40`, pour que les murs
qu'un Évider ou un Soustraire laisse derrière lui se distinguent. L'identifiant reste le même quand l'objet bouge et d'un
enregistrement à l'autre.

### Nommer les choses { #naming-things }

Sélectionnez un objet et appuyez sur F2, ou faites un clic droit dessus dans l'Outliner ou dans une vue et choisissez
*Renommer*. La ligne devient un champ de texte. Entrée garde le nom, Échap annule, et un nom vide ramène le nom par défaut.

![Une ligne de brush dans l'Outliner transformée en champ de texte avec « front wall » saisi](../assets/organizing/outliner-rename.png)

Les noms sont enregistrés avec la carte. Les entités gardent leur `targetname` à part de ce nom, donc renommer ne change
jamais ce qu'une sortie atteint. Les morceaux qu'un Clip, une opération CSG ou une conversion laisse derrière eux gardent
le nom, et nommer un objet dans un groupe lié le nomme dans toutes les copies.

(Les calques, les groupes et les ensembles de scatter gardent toujours un nom, donc pour eux un nom vide ne change rien.
Renommer un ensemble de scatter renomme aussi le calque que l'outil Scatter a créé pour lui.)

### Choisir des objets dans l'Outliner { #picking-objects-in-the-outliner }

Ctrl+clic sur une ligne l'ajoute à la sélection ou la retire. Shift+clic sélectionne toutes les lignes entre la dernière
sur laquelle vous avez cliqué et celle-ci, et Ctrl+Shift+clic ajoute cette plage à ce qui est déjà sélectionné.
L'Outliner est souvent l'endroit le plus simple pour choisir des objets cachés derrière d'autres dans les vues.

### Garder un calque hors de Godot { #keeping-a-layer-out-of-godot }

Pour une géométrie de référence qui ne doit jamais arriver dans le jeu, faites un clic droit sur son calque et choisissez
*Basculer l'omission à l'export*. Le calque affiche alors *[omis]* après son nom. Le masquer ne suffit pas, Godot construit
quand même les objets masqués.

![Le menu du clic droit du calque de référence Blockout avec Basculer l'omission à l'export survolé](../assets/organizing/layer-menu-omit.png)

## Groupes { #groups }

Un groupe garde des objets ensemble pour qu'ils se sélectionnent et se déplacent d'un bloc, comme les brushes d'une table.
Tout ce qui concerne les groupes se trouve sous *Édition > Groupes*.

![Le menu Édition avec Groupes ouvert et Dupliquer en lien survolé](../assets/organizing/menu-groups.png)

| Entrée | Résultat |
| --- | --- |
| Ctrl+G, Ctrl+Shift+G | Grouper la sélection, la dégrouper |
| Clic | Sélectionner l'objet à l'intérieur du groupe |
| Double clic | Sélectionner tout le groupe |

**Dupliquer en lien** (Ctrl+Shift+D) crée une copie qui reste synchronisée avec son original, si bien que corriger un
cadre de fenêtre les corrige tous. Les copies liées affichent *(lié)* dans l'Outliner, et un clic sur l'une sélectionne
tout le groupe, puisque ses copies se modifient d'un bloc. *Édition > Groupes > Délier les groupes* rompt le lien quand une
copie doit différer.

## Prefabs { #prefabs }

Pour des pièces que vous réutilisez d'une carte à l'autre, sélectionnez-les et choisissez *Édition > Prefabs > Créer depuis
la sélection…*. Cela enregistre la sélection dans son propre `.tdm` et met une instance à sa place. Chaque instance se met
à jour quand le fichier du prefab change.

![Le menu Édition avec Prefabs ouvert et Créer depuis la sélection survolé](../assets/organizing/menu-prefabs.png)

> [!NOTE]
> Enregistrez la carte une fois avant de créer un prefab. Les chemins de prefab sont stockés relativement à la carte, donc
> l'éditeur refuse tant que la carte n'a pas de fichier.

Pour des pièces que vous collez une fois puis modelez librement, utilisez plutôt la [bibliothèque de prefabs](prefab-library.md).

Placer un prefab deux fois donne à deux entités le même targetname. Le fixup d'une instance règle ça : avec un fixup `p1`,
chaque nom à l'intérieur reçoit ce préfixe, donc `door` devient `p1-door` et chaque copie garde son câblage à part. Le
fichier de carte et la construction Godot connaissent le fixup, mais l'éditeur n'a pas encore de champ pour le régler.

## Cordon { #cordon }

Un cordon réduit l'éditeur à une seule partie d'une grande carte.

1. Sélectionnez ce sur quoi vous voulez travailler.
2. Choisissez *Affichage > Cordon > Définir le cordon depuis la sélection*, ou cliquez sur **Cordon** dans l'Inspecteur.

    ![Le menu Affichage avec Cordon ouvert et Définir le cordon depuis la sélection survolé](../assets/organizing/menu-cordon.png)

3. Tout ce qui ne touche pas la boîte jaune autour de votre sélection est maintenant masqué et ne peut plus être
    sélectionné, et la barre d'état indique *cordon*.

    ![Avant : toute la carte avec la maison, la pelouse, un mur de jardin, un pilier et un grand bloc de référence. Après : seulement la maison dans une boîte jaune et la pelouse qui la touche](../assets/organizing/cordon-before-after.png)

*Affichage > Cordon > Cordon activé* le désactive et le réactive, et *Effacer le cordon* le supprime. Le cordon est
enregistré avec la carte.

Il ne change que ce que l'éditeur montre, Godot construit toujours la carte entière. Pour confier une zone à un autre
outil, *Fichier > Exporter > .map, cordon seulement…* n'écrit que les objets qui sont dedans.

## Problèmes { #issues }

Le panneau Problèmes, l'onglet à côté de l'Outliner et de l'Historique, vérifie la carte pendant que vous travaillez. Il
liste les problèmes qui casseraient ou auraient un mauvais aspect dans Godot, comme des brushes sans volume, des faces sans
matériau, ou des sorties dirigées vers un targetname que rien ne porte.

![Le panneau Problèmes avec une erreur pour une cible manquante, trois avertissements pour un brush en double, une entité sans définition et deux faces qui z-fightent, et une ligne d'information pour un groupe vide, la plupart avec un bouton Corriger](../assets/organizing/issues-panel.png)

Cliquez sur un problème pour sélectionner et cadrer l'objet. Beaucoup ont un bouton **Corriger**, et **Tout corriger** dans
l'en-tête corrige tous ceux qu'il peut.

Un avertissement disant qu'une face chevauche une face d'un autre brush sur le même plan signifie que les deux
scintilleront dans Godot. Éloignez une face du plan ou rognez le chevauchement.

Un avertissement disant qu'un modèle n'existe pas signifie que le `model` d'un accessoire nomme un fichier absent, donc
l'accessoire est construit sans maillage. Quand un fichier du même nom avec une autre extension existe, `chair.gltf` pour
`chair.glb`, l'avertissement le nomme.

Un avertissement disant que quelque chose exécute du GDScript depuis la carte signifie que la carte porte son propre code,
qui s'exécute dès que la scène se joue dans Godot. Vous en avez un pour chaque `logic_script` avec une `source` ou une
`expression`, chaque sortie ou `logic_call` qui peut lui passer du code, comme une sortie qui lui envoie `source`, et chaque
prefab qui contient l'un d'eux. Lisez-les avant de jouer une carte faite par quelqu'un d'autre.

Les avertissements qui commencent par « Godot » viennent du dernier build dans Godot, après un enregistrement ou
*Construire dans Godot* : les classes qu'il a construites sans leur définition, et les autres avertissements de ce build.
Voir [Entités qui ne font rien](../godot/entities-troubleshooting.md).

## Historique { #history }

Le panneau Historique liste chaque modification, la plus récente en haut. Cliquez sur une entrée plus ancienne pour annuler
jusqu'à elle, cette entrée comprise. Les modifications annulées restent estompées au-dessus de *actuel*, cliquez sur l'une
d'elles pour refaire jusqu'à elle.

![Le panneau Historique avec Renommer et Grouper estompés au-dessus du repère actuel et l'entrée Évider survolée](../assets/organizing/history-panel.png)

<div class="td-mcp" markdown>

## MCP { #mcp }

Un agent ou un script qui pilote l'éditeur via le [serveur MCP](../mcp.md) lance les mêmes vérifications que le panneau
Problèmes avec l'outil `validate_map`, qui les nomme par code, comme `coplanar_faces` et `missing_model`. L'action
`insert_prefab` de `run_action` accepte un `fixup`, donc un agent peut donner son fixup à une instance en la plaçant.

</div>
