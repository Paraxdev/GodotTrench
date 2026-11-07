---
source: 18fb3370f830b9fda2728d12b95c3b5cef06be64
---

# Premiers pas { #getting-started }

Étapes 1 à 3 : la première configuration. Si vous avez déjà un projet Godot configuré avec TrenchDot, vous pouvez les sauter.

Étapes 4 et 5 : créer la première pièce dans TrenchDot et la voir dans Godot.

## Ce qu'il vous faut { #what-you-need }

* Godot 4.7 ou plus récent, la version .NET si vous voulez écrire des entités en C#
* La [version de TrenchDot](https://github.com/Paraxdev/TrenchDot/releases/tag/beta) la plus récente

## Vérifier que TrenchDot voit l'exécutable Godot { #ensure-trenchdot-can-see-the-godot-executable }

Assurez-vous que TrenchDot voit votre installation de Godot. Pour vérifier que tout est bien détecté, ouvrez
*Fichier > Préférences… > Exécutable Godot*.

Si TrenchDot a trouvé Godot tout seul, la ligne sous le champ de saisie indique déjà « Utilise » suivi du chemin. Si elle
indique « Introuvable » à la place, il faut aller chercher l'exécutable sur votre disque.

Pour cela, cliquez sur le bouton « … » à côté du champ de saisie

![Le bouton … à côté du champ Exécutable Godot](assets/getting-started/godot-path-browse.png)

et choisissez l'exécutable dont le nom commence par « Godot_v ».

Si vous avez choisi le bon exécutable, la ligne doit indiquer « Utilise » suivi du chemin que vous venez de choisir.

![La ligne sous le champ de saisie, qui indique Utilise et le chemin de l'exécutable Godot](assets/getting-started/godot-path-using.png)

## 1. Créer un projet dans Godot { #1-create-a-project-in-godot }

Ouvrez Godot et cliquez sur « Create » dans le coin supérieur gauche.

![Le gestionnaire de projets de Godot, le bouton Create avec une icône plus dans le coin supérieur gauche](assets/getting-started/godot-create-project.png)

Choisissez un nom de projet, dans ce tutoriel ce sera TrenchDotTest, et (si vous voulez) un dossier où le projet doit
vivre.

Cliquez ensuite sur « Create ».

## 2. Ouvrir le projet dans TrenchDot { #2-opening-the-project-within-trenchdot }

Choisissez *Godot > Ouvrir un projet Godot…*

![Le menu Godot de TrenchDot avec Ouvrir un projet Godot… en surbrillance](assets/getting-started/menu-open-godot-project.png)

ou le bouton **Ouvrir un projet Godot…** en haut à droite de la barre d'outils

![Le bouton Ouvrir un projet Godot… en haut à droite de la barre d'outils de TrenchDot, qui affiche le nom du projet une fois qu'un projet est ouvert](assets/getting-started/toolbar-open-godot-project.png)

et choisissez le dossier du projet ou n'importe quel dossier à l'intérieur. L'éditeur le rouvre au prochain démarrage.

## 3. L'écran d'accueil { #3-the-welcome-screen }

Dès que TrenchDot détecte qu'il s'agit d'un nouveau projet Godot, il affiche la fenêtre *Configurer ce projet*.

![La fenêtre Configurer ce projet avec les sections de l'addon, des entités Gameplay et du contenu prêt à l'emploi](assets/getting-started/setup-window.png)

Vous pouvez y télécharger l'addon, dont Godot a besoin pour transformer vos cartes en scènes et pour parler à TrenchDot,
et du contenu en plus si vous le souhaitez. Vous pouvez bien sûr aussi commencer sans rien.

Quelques entités de base sont toujours incluses, comme un simple `info_player_start` et `light`.

1. Installer l'addon

    Cliquez sur le bouton « Installer l'addon » tout en haut pour télécharger le fork de FuncGodot de TrenchDot dans
    votre projet, c'est indispensable pour TrenchDot.
    (Remarque : il est téléchargé depuis les versions GitHub. Si vous n'avez pas d'accès à Internet, téléchargez
    d'abord `func_godot-trenchdot-addon.zip` et extrayez-le à la racine du projet pour l'installer à la main.)

    Une fois cela fait, cliquez sur « Activer le plugin » pour que Godot le charge.
    (Remarque : fermez d'abord Godot s'il a le projet ouvert, sinon Godot réécrit sa propre copie de `project.godot` et
    le plugin reste désactivé.)

2. Installer les entités Gameplay

    Cliquez sur **Installer les entités Gameplay** sous l'addon.

    ![La section Entités Gameplay de la fenêtre Configurer ce projet avec le bouton Installer les entités Gameplay](assets/getting-started/setup-gameplay-entities.png)

    Les portes, boutons, ascenseurs, déclencheurs, props et spawners viennent avec ce pack. Sans lui, Godot les construit
    comme de simples formes qui n'ont pas de collision et ne font rien. Les aides pour portes et déclencheurs du menu
    Gameplay installent le pack d'elles-mêmes la première fois que vous les utilisez, mais l'installer maintenant, avant
    que Godot n'ouvre le projet pour la première fois, garantit qu'aucune scène n'est jamais construite sans lui.

3. (Facultatif) Du contenu en plus

    TrenchDot propose du contenu facultatif que vous pouvez précharger : des textures et des modèles CC0 de Poly Haven et
    d'ambientCG, ainsi que des modèles faits pour TrenchDot. Les licences et la liste de la provenance de chaque élément
    sont incluses.

    Si vous ne voulez aucun contenu en plus, choisissez « Rien de plus », les entités minimales sont quand même incluses.

4. Ouvrir une fois le projet dans Godot

    Quand le plugin est activé, ouvrez le projet dans Godot (la fenêtre a pour cela un bouton « Ouvrir le projet dans
    l'éditeur Godot »). L'addon crée alors un fichier appelé `trenchdot_game.json` à la racine du projet. C'est de là que
    TrenchDot apprend à connaître vos textures et vos entités, et il le charge en moins d'une seconde.

    L'addon le réécrit chaque fois que des fichiers changent, vous n'avez donc jamais à le modifier à la main. Tant qu'il
    n'existe pas, la barre d'état indique qu'il n'y a pas de `trenchdot_game.json`, et l'éditeur propose les entités de
    base, plus les entités Gameplay si vous les avez installées.

Pour ajouter vos propres entités, voyez [Entités personnalisées en GDScript](gameplay/custom-entities.md) et
[Entités en C#](gameplay/csharp.md). [Configuration du projet](godot/project-setup.md) explique les ressources qui les
listent.

## 4. Construire une pièce { #4-build-a-room }

L'éditeur a 4 vues principales : une vue 3D dans laquelle vous pouvez voler comme dans Godot, et 3 vues en fil de fer,
de dessus, de face et de côté.

Les brushes et les opérations dépendent de la taille de la grille. Dans la barre d'outils, vous pouvez régler la taille
de la grille (plus petite, elle est plus fine).

![Le réglage de la taille de la grille dans la barre d'outils, réglé sur 16](assets/getting-started/toolbar-grid-size.png)

Sur un niveau vide, un petit texte d'information explique comment dessiner. Vous pouvez le masquer avec « Masquer les
astuces » ou simplement commencer à dessiner, il disparaît dès que la carte contient quelque chose.

![Une nouvelle carte : la carte dans la vue 3D dit de glisser dans la vue de dessus pour tracer la première boîte, et la vue de dessus montre une étiquette jaune « Glissez ici pour tracer votre première boîte »](assets/getting-started/empty-map.png)

1. Dessiner votre première boîte

    Avec l'outil Sélectionner (Q), glissez sur un espace vide dans n'importe quelle vue.
    Vous devriez voir une boîte se dessiner. En gardant le clic gauche enfoncé, bougez la souris pour donner sa forme au
    rectangle.

    ![Les quatre vues avec la première boîte dessinée, la vue de dessus affichant sa taille, 512 par 256](assets/getting-started/draw-first-box.png)

2. Ajouter de la hauteur

    Dans la vue de côté, placez la souris sur l'arête supérieure de la boîte jusqu'à ce qu'elle devienne jaune, gardez le
    clic gauche enfoncé, puis montez la souris jusqu'à ce que la boîte fasse 256 unités de haut.

    ![La vue de côté avec l'arête supérieure de la boîte tirée vers le haut, 256 affiché au-dessus et à côté](assets/getting-started/side-view-depth.png)

3. L'évider

    Appuyez sur Ctrl+Shift+K, ou utilisez le bouton Évider de la barre d'outils.

    ![La barre d'outils avec le bouton Évider survolé et son infobulle](assets/getting-started/toolbar-hollow.png)

    La boîte est maintenant découpée en murs, un sol et un plafond, ce qui est idéal pour faire une petite pièce ou un
    couloir. Avec les commandes de vol (maintenez le bouton droit de la souris et utilisez WASD), entrez en volant dans
    la boîte creuse.

4. Percer une porte

    Avec Échap, vous désélectionnez les brushes sélectionnés. Une fois que plus rien n'est sélectionné, dessinez dans la
    vue de face un rectangle vertical plus petit à l'intérieur de la boîte, en forme de porte.

    ![Le brush de la porte dans la boîte creuse, 64 de large et 96 de haut dans la vue de face](assets/getting-started/doorway-brush.png)

    Puis, dans la vue de côté, étirez un côté du brush de la porte pour lui donner de la profondeur, jusqu'à traverser
    tout le mur.

    ![Un côté du brush de la porte tiré dans la vue de côté pour qu'il traverse le mur](assets/getting-started/doorway-extend.gif)

    Maintenant, avec le brush sélectionné, appuyez sur Ctrl+K ou utilisez le bouton Soustraire de la barre d'outils.

    ![La barre d'outils avec le bouton CSG Soustraire survolé et son infobulle](assets/getting-started/toolbar-subtract.png)

    Vous devriez voir le brush disparaître, et la porte est découpée dans le mur.

    ![Le mur dans la vue 3D avec la porte découpée](assets/getting-started/doorway-cut.png)

5. Le texturer

    Vous devriez avoir 5 textures incluses pour le prototypage. Pour appliquer une texture à une seule face (par exemple
    le cadre de la porte), glissez la texture de l'onglet Matériaux sur la face (maintenez Shift pour couvrir tout le
    brush).

    ![Glisser dev/blue de l'onglet Matériaux sur une face, avec l'aperçu du dépôt](assets/getting-started/texture-drop.png)

6. Enregistrer la carte

    Pour voir les changements dans Godot, il faut enregistrer la carte. Appuyez sur Ctrl+S, ou utilisez *Fichier > Enregistrer*.

    La boîte de dialogue d'enregistrement s'ouvre dans le dossier de votre projet Godot. Enregistrez-la où vous voulez à
    l'intérieur du projet, un dossier `maps` à la racine du projet garde les choses en ordre, par exemple
    `res://maps/room.tdm`.
    (Remarque : le nom du dossier vous appartient, mais gardez la carte dans le projet, pour que Godot puisse la charger
    par son chemin `res://`.)

    Une fois enregistrée, elle apparaît sous *Fichier > Cartes du projet*.

    ![Le menu Fichier avec Enregistrer et Cartes du projet, qui liste room sous res://maps](assets/getting-started/file-menu-maps-in-project.png)

## 5. La voir dans Godot { #5-see-it-in-godot }

Dans Godot, un niveau est une scène : un arbre de nœuds, enregistré dans un fichier `.tscn`, où chaque nœud est une chose
comme un mesh, une lumière ou une caméra. Godot écrit les chemins à l'intérieur du projet avec `res://`, donc
`res://maps/room.tdm` est le fichier `maps/room.tdm` dans le dossier de votre projet. La carte est transformée en nœuds
par un nœud de l'addon, `FuncGodotMap`.

1. Dans l'éditeur Godot, choisissez *Scene > New Scene* et cliquez sur **3D Scene** dans le panneau Scene à gauche.
2. Appuyez sur Ctrl+A (*Add Child Node*), cherchez `FuncGodotMap` et ajoutez-le.
3. Avec le nouveau nœud sélectionné, réglez *Local Map File* sur votre `.tdm` dans l'Inspector propre à Godot, le panneau
   à droite de l'éditeur Godot. Il fonctionne comme l'Inspecteur de TrenchDot mais appartient à Godot.
4. Appuyez sur **Build Map** en haut de cet Inspector, puis enregistrez la scène avec Ctrl+S.

Les brushes deviennent des meshes avec collision et les entités deviennent des nœuds avec leurs scripts, enregistrés
avec la scène. [Construire des cartes](godot/building.md) explique les autres réglages du nœud. Si une entité sort comme
un mesh nu sans collision, la barre d'état de TrenchDot en donne la raison, voyez
[Les entités qui ne font rien](godot/entities-troubleshooting.md).

Laissez cette scène ouverte dans Godot et chaque enregistrement dans TrenchDot la reconstruit, tout de suite tant que son
onglet est l'onglet courant, ou quand vous y revenez. Enregistrez la scène dans Godot pour garder le résultat. Si un
enregistrement ne reconstruit pas, la barre d'état en donne la raison, voyez
[Rebuild on save](godot/live.md#rebuild-on-save).

> [!WARNING]
> Une construction supprime d'abord les enfants du `FuncGodotMap`. Mettez vos propres nœuds à côté du nœud de la carte,
> ou dans un [overlay](godot/overlays.md).

## Essayer le projet de démo { #try-the-demo-project }

Le dossier `godot` du dépôt, aussi dans la version sous le nom `trenchdot-demo-project.zip`, est un projet avec l'addon
installé. Sa scène principale joue les cartes de présentation, enregistrées dans `godot/demo/maps/showcase`. Ouvrez le
projet dans l'éditeur et choisissez-en une sous *Fichier > Cartes du projet* pour voir comment une carte terminée est
assemblée.

Le choix de la démo dans *Godot > Ajouter du contenu au projet…* met les mêmes cartes et scènes dans votre propre
projet. Dans Godot, ouvrez `res://demo/demo.tscn` et appuyez sur F6 pour les jouer. Ouvertes dans TrenchDot depuis votre
projet, elles s'affichent sans leurs textures, parce qu'elles utilisent le dossier de textures de la démo
`res://demo/textures` plutôt que celui de votre projet. Le projet de démo les montre telles qu'elles ont été construites.

| Touche | Action |
| --- | --- |
| WASD, Espace, Shift | Marcher, sauter, courir |
| E | Utiliser les portes et les boutons |
| 1 à 5 | Passer d'une carte de présentation à l'autre |
| F3 | Afficher l'overlay de débogage des E/S, un journal de chaque input et output au moment où il se déclenche, avec les volumes déclencheurs dessinés |

## Ensuite { #next }

Pour que la pièce fasse quelque chose, lisez [Comment fonctionnent les E/S d'entités](gameplay/io.md) et construisez
[Un bouton ouvre une porte](tutorials/button-door.md).

Votre propre joueur a besoin de deux choses pour les entités du pack. Les déclencheurs ne réagissent qu'aux corps du
groupe Godot `player` sur la couche physique 1, mettez donc votre joueur là. Les portes et les boutons bougent quand le
code de votre joueur appelle `use()` sur celui qu'il regarde, voyez
[Laisser le joueur appuyer dessus](tutorials/button-door.md#let-the-player-press-it).

<div class="td-mcp" markdown>

## MCP { #mcp }

La question du contenu ne s'ouvre pas d'elle-même dans un éditeur piloté par un client MCP : le premier appel d'outil
autre que `get_state`, `screenshot` et `simulate_input` la ferme, et on ne pose pas la question pour un projet ouvert
avec `open_project`. `project_content` ajoute le même contenu, voyez [MCP](mcp.md).

</div>
