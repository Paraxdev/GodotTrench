---
source: 7982e4d62890afb0393dc32addda8d5b4d33f84a
---

# Interface et navigation { #interface-and-navigation }

TrenchDot suit TrenchBroom et Hammer, donc si vous avez utilisé l'un des deux, la plupart de vos habitudes restent. Sinon,
cette page vous fait faire le tour, et [Venir de Unity ou de Blender](coming-from-unity-blender.md) fait le lien avec les
idées que vous connaissez déjà.

## La fenêtre { #the-window }

![L'éditeur avec une petite pièce : l'Outliner à gauche, les vues 3D, de dessus, de face et de côté au milieu, l'Inspecteur à droite et le panneau Matériaux en bas](../assets/interface/window.png)

Les menus sont en haut. Une commande se trouve sous le type d'objet qu'elle modifie, si bien que la CSG est sous *Brush* et
la sculpture de terrain sous *Terrain*.

Vous ne trouvez pas quelque chose ? Appuyez sur Ctrl+Shift+P (ou F1) pour la palette de commandes et tapez son nom. Les
commandes au nom jargonnant, comme *Évider* ou *Verrouillage UV*, s'expliquent quand vous les survolez, et la palette
montre la même ligne sous sa liste. Esc ou un clic à l'extérieur la ferme sans rien lancer.

![La palette de commandes avec hollow saisi, Brush: Évider choisi avec son raccourci, et la ligne qui l'explique en dessous](../assets/interface/command-palette.png)

Juste sous la barre d'outils se trouve la **barre d'options de l'outil**. Elle contient les réglages de l'outil actif, ou
une ligne sur la façon d'utiliser un outil qui n'en a pas. Survolez le nom de l'outil à cet endroit, ou son bouton dans la
barre d'outils, pour une courte description.

![La barre d'options de l'outil avec l'outil Sélectionner survolé, son infobulle décrit l'outil et finit par F1 : manuel](../assets/interface/tool-options-tooltip.png)

### L'encart de la carte vide { #the-empty-map-card }

Sur une carte neuve et vide, un encart dans la vue 3D vous dit par où commencer : faites glisser dans la vue de dessus pour
tracer votre première boîte, et la vue de dessus marque l'endroit avec une étiquette jaune. En dessous, elle explique la
caméra 3D et renvoie vers le [guide de démarrage](../getting-started.md). Quand le projet ouvert a des cartes, **Ouvrir une
carte de ce projet** les liste, pour que vous puissiez en ouvrir une terminée à la place.

![L'encart de la carte vide dans la vue 3D avec Guide de démarrage, Ouvrir une carte de ce projet et Masquer les astuces, à côté de l'étiquette jaune dans la vue de dessus](../assets/interface/empty-map-card.png)

L'encart disparaît dès que la carte contient quelque chose. **Masquer les astuces** désactive les astuces, et *Astuces sur
une carte vide* dans *Fichier > Préférences* les ramène.

### Panneaux { #panels }

| Panneau | Emplacement | À quoi il sert |
| --- | --- | --- |
| Outliner | Gauche | La carte sous forme d'arbre de [calques, groupes](organizing.md) et objets |
| Historique | Gauche | Chaque modification. Cliquer sur l'une l'annule avec tout ce qui suit, cliquer sur une modification grisée la rétablit jusque-là |
| Problèmes | Gauche | Les problèmes trouvés pendant que vous travaillez, comme des faces sans matériau. Cliquez sur l'un pour sélectionner l'objet, la plupart proposent une correction |
| Inspecteur | Droite | Les propriétés de la sélection, comme les clés d'une entité ou la position et la taille exactes d'un brush |
| Entités | Droite | Les classes d'entités à faire glisser dans une vue |
| Éditeur UV | Droite | [Placement exact des textures](uv-editor.md) |
| Scatter | Droite | Les modèles que peint l'outil [Scatter](scatter.md) |
| Matériaux | Bas | Les textures de votre projet |
| Modèles | Bas | Les [fichiers de modèle](models.md) à placer |
| Prefabs | Bas | Des pièces prêtes à coller issues de la [bibliothèque de prefabs](prefab-library.md) |
| Logique | Fermé au départ | Le [câblage des entités sous forme de graphe](../gameplay/logic-graph.md), pour connecter des entités et prévisualiser ce qu'une sortie déclenche |
| Référence | Fermé au départ | Du code pour utiliser une classe d'entité depuis vos scripts |

Vous avez fermé un panneau par erreur ? Rouvrez-le depuis *Affichage > Panneaux*. *Réinitialiser la disposition*, en bas du
même menu, remet tout là où c'était au départ.

![Le menu Affichage avec Panneaux ouvert, les panneaux affichés cochés et Réinitialiser la disposition en bas](../assets/interface/view-panels-menu.png)

## L'aide et le manuel { #help-and-the-manual }

Les infobulles expliquent les boutons, les onglets et les réglages. Pour en savoir plus, pointez un outil, un onglet de
panneau, une entrée de menu ou une section de l'Inspecteur et appuyez sur **F1**, et la page correspondante de ce manuel
s'ouvre. Un Ctrl+clic sur un outil dans la barre d'outils (Cmd+clic sous macOS) fait la même chose. Une infobulle qui finit
par *F1 : manuel* a une page derrière elle, sur tout le reste F1 ouvre la palette de commandes.

*Aide > Manuel* ouvre le début du manuel.

![Le menu Aide avec Manuel survolé](../assets/interface/help-menu.png)

Une fois que vous vous repérez, décochez *Texte d'aide* dans *Fichier > Préférences* pour masquer les courtes lignes
d'indication dans la barre d'options de l'outil et les panneaux. Leurs infobulles restent.

(Remarque : les liens s'ouvrent dans votre navigateur web. Si aucun navigateur ne s'ouvre, la barre d'état le dit et le lien
est copié dans le presse-papiers.)

## Vues { #views }

Les vues sont 3D, Dessus, Face et Côté. Les vues 2D regardent droit le long d'un axe, ce qui facilite l'alignement. Avec
quelque chose de sélectionné, elles impriment sa largeur au-dessus et sa hauteur à côté.

![La vue de face avec un brush en boîte sélectionné, 96 imprimé au-dessus et 48 à côté](../assets/interface/front-view-size-labels.png)

Un clic dans une vue 2D sélectionne l'objet le plus proche le long de la vue, donc dans la vue de dessus un plafond cache
les murs qui sont dessous. Cliquez de nouveau au même endroit dans la seconde et demie qui suit, ou faites Alt+clic à tout
moment, pour passer à l'objet suivant sous le curseur.

| Faites ceci | Pour |
| --- | --- |
| Faire glisser le point où les séparateurs se croisent | Redimensionner les quatre vues d'un coup |
| Shift+Space | Agrandir la vue sous le pointeur, de nouveau pour restaurer |
| *Affichage > Vues* | Passer entre une, deux et quatre vues, ou rouvrir une vue fermée. *Une vue* garde la vue où vous avez travaillé en dernier, la vue 3D au début, et pendant qu'une seule vue est affichée, choisir le nom d'une autre vue y passe |

![Le menu Affichage avec Vues ouvert : les quatre vues cochées, Une vue, Deux vues et Quatre vues, et Agrandir la vue sur Shift+Space](../assets/interface/view-views-menu.png)

## Onglets { #tabs }

Chaque carte ouverte reçoit un onglet, et la barre d'onglets apparaît sous la barre d'options de l'outil dès que deux
sont ouverts. Ctrl+Shift+N (ou le bouton **+**) ouvre un nouvel onglet, Ctrl+Tab change d'onglet et Ctrl+W en ferme un.

![La barre d'onglets avec room.tdm et untitled.tdm ouverts et le bouton + survolé, qui montre Nouvel onglet](../assets/interface/map-tabs.png)

*Nouvelle carte* et l'import s'ouvrent dans un nouvel onglet quand la carte actuelle a des modifications non enregistrées.
Fermer un onglet modifié demande d'enregistrer, d'abandonner ou d'annuler, et quitter liste toutes les cartes non
enregistrées.

## Caméra { #camera }

| Action | Vue 3D | Vues 2D |
| --- | --- | --- |
| Glisser avec le clic droit | Regarder autour. WASD déplace, Q et E descendent et montent, Shift triple la vitesse | Panoramique |
| Glisser avec le clic du milieu | Panoramique | Panoramique |
| Molette | Avancer et reculer, plus vite quand la surface droit devant est loin | Zoom vers le curseur |
| Alt+glisser avec le clic gauche | Orbite, sauf si le glisser commence sur la sélection, qui bouge alors à la verticale | |

WASD ne fait voler que tant que vous maintenez le bouton droit, sinon ces touches sont des raccourcis. Vous avez oublié les
commandes ? Survolez l'onglet d'une vue. Tant que vous n'avez pas regardé autour de vous une première fois, la vue 3D les
montre aussi sous son nom quand le pointeur est dessus (**Masquer les astuces** sur l'encart de la carte vide désactive
cela).

**F** cadre la sélection dans chaque vue, ou toute la carte quand rien n'est sélectionné. Un cran de molette zoome une vue
2D du même facteur à n'importe quel zoom, donc passer d'un grand terrain entier à une seule pièce prend environ 20 crans,
et **F** sur la pièce est plus rapide.

Ctrl+Shift+1 à 9 enregistre la caméra 3D comme signet et Ctrl+1 à 9 y retourne. Les signets sont enregistrés dans la carte,
et l'annulation n'y touche pas.

## Ombrage { #shading }

F4 fait passer la vue 3D par Texturé, Plat, Aperçu éclairé et Fil de fer, et la vue imprime le mode en haut à gauche. Une
fois que la carte a un [précalcul](light-baking.md), **Éclairage précalculé** vient après Aperçu éclairé. F3 saute
directement à **Aperçu éclairé** et revient à Texturé, sans passer par les autres.

![La vue 3D qui passe par l'ombrage texturé, plat, éclairé et fil de fer](../assets/interface/shading-cycle.gif)

L'Aperçu éclairé montre les lumières de la carte, les ombres du soleil, le ciel et le brouillard sans construire dans
Godot. Il ne dessine que les 64 lumières ponctuelles et spots les plus fortes, si bien qu'une carte qui en a beaucoup plus
peut paraître plus sombre ici que dans Godot.

## Thème { #theme }

TrenchDot existe en un aspect sombre et un aspect clair. Une installation neuve démarre en sombre.

1. Ouvrez **Fichier > Préférences**. **Thème** est la première ligne, en haut de la fenêtre.
2. Cliquez sur la liste déroulante et choisissez **Sombre**, **Clair** ou **Système**.

    ![La liste déroulante Thème dans les Préférences, ouverte avec Sombre, Clair et Système](../assets/interface/preferences-theme.png)

3. L'éditeur entier change à l'image suivante, sans redémarrage. Le choix est enregistré avec vos autres préférences.

    ![L'éditeur avec le thème clair, avec la barre de menus, l'Outliner, la vue 3D et la vue de dessus](../assets/interface/theme-light-window.png)

**Système** suit le mode clair ou sombre des applications de votre système d'exploitation et change avec lui. Tant qu'il
est choisi, la liste déroulante affiche *Système (Clair)* ou *Système (Sombre)* pour que vous voyiez ce que votre système a
donné. Là où le système ne signale pas de mode, l'éditeur reste sombre.

Vous pouvez aussi le choisir dans **Affichage > Thème**, juste sous *Échelle de l'interface*, ou dans la palette de
commandes avec *Affichage: Thème: Sombre*, *Clair* ou *Système*. Aucune touche n'est assignée par défaut, mais vous pouvez
donner une touche à chacun dans l'éditeur de raccourcis.

En Clair, les vues 2D (dessus, face et côté) deviennent du papier, un fond gris clair avec des lignes de grille et des
arêtes de brush sombres, pour que les lignes que vous tracez se lisent toujours d'un coup d'œil. La vue 3D garde son
entourage sombre dans les deux thèmes, tout comme les canevas de texture de l'éditeur UV et de l'éditeur de hotspots. Une
texture a le même aspect dans l'un ou l'autre thème, et un entourage clair changerait la façon dont vous jugez ses
couleurs.

<div class="td-mcp" markdown>

Via le [serveur MCP](../mcp.md), `set_editor` avec `"theme": "light"` change le thème. Les valeurs sont `dark`, `light` et
`system`, et une valeur inconnue est une erreur qui ne change rien. `get_state` signale le choix dans `ui.theme` et ce qui
est à l'écran dans `ui.theme_active`.

</div>

## Langue { #language }

Les menus, les panneaux, les infobulles et les messages d'état existent en anglais, allemand, français, espagnol,
portugais, russe, chinois simplifié et japonais. Une installation neuve démarre en anglais.

1. Ouvrez **Fichier > Préférences**. **Langue** est la deuxième ligne, juste sous **Thème**.
2. Cliquez sur la liste déroulante et choisissez votre langue. Chaque langue est listée sous son propre nom, vous pouvez
    donc trouver la vôtre même quand l'éditeur est dans une langue que vous ne lisez pas.

    ![La liste déroulante Langue dans les Préférences, ouverte avec English, Deutsch, Français, Español, Português, Русский, 简体中文 et 日本語](../assets/interface/preferences-language.png)

3. L'éditeur entier change tout de suite, sans redémarrage. Le choix est enregistré avec vos autres préférences.

    ![Le menu Fichier en allemand](../assets/interface/language-file-menu-de.png)

Tant qu'une autre langue que l'anglais est active, la ligne affiche par exemple *Sprache (Language)*, pour que vous
retrouviez toujours votre chemin. Vous pouvez aussi choisir une langue dans la palette de commandes avec *Affichage:
Langue: Deutsch*, et la même commande marche dans chaque langue, puisque la palette cherche aussi dans les noms anglais.
Aucune touche n'est assignée par défaut, mais vous pouvez donner une touche à chaque langue dans l'éditeur de raccourcis.

Les traductions sont écrites par une IA et aucun locuteur natif ne les a encore relues, une note sous la ligne le dit. Un
texte qui n'a pas encore de traduction s'affiche en anglais, tout comme une langue que cette version ne connaît pas. Si une
traduction sonne faux, vous pouvez la corriger dans [le fichier de langue correspondant](https://github.com/Paraxdev/TrenchDot/tree/main/crates/td_editor/locales),
voir [Traductions](../development.md#translations).

> [!NOTE]
> Le manuel suit la langue de l'éditeur. Quand une langue est choisie, **F1** sur un outil, un panneau ou un menu, et les
> entrées du menu **Aide**, ouvrent cette page du manuel dans votre langue. Le bouton de langue dans l'en-tête du manuel
> change la langue ici. Une page qui n'est pas encore traduite, ou qui a changé depuis sa traduction, s'affiche en anglais
> avec une note en haut. Les captures d'écran montrent l'éditeur en anglais, cherchez donc la même entrée dans votre
> langue, ou tapez le nom anglais dans la palette de commandes. Les noms de classe d'entité, les clés de carte, les formats
> de fichier, le journal et le [serveur MCP](../mcp.md) restent en anglais dans toutes les langues.

<div class="td-mcp" markdown>

Via le [serveur MCP](../mcp.md), `set_editor` avec `"language": "de"` change la langue. Les valeurs sont `en`, `de`, `fr`,
`es`, `pt`, `ru`, `zh-Hans` et `ja`, quelle que soit la casse, et une valeur inconnue est une erreur qui les nomme toutes
et ne change rien. `get_state` signale le choix dans `ui.language`. Les noms d'outils, les réponses et les erreurs restent
en anglais dans toutes les langues. Un éditeur lancé avec `--default-prefs` est en anglais, ce sur quoi s'appuient les
tests e2e.

</div>

## Échelle de l'interface { #interface-scale }

Ctrl+= et Ctrl+- changent l'échelle de l'interface et Ctrl+0 la réinitialise. Vous trouvez la même chose sous
*Échelle de l'interface* en haut de *Fichier > Préférences*. Si l'écran annonce une mauvaise mise à l'échelle, décochez
*suivre l'écran* juste en dessous.

![Le haut de la fenêtre Préférences : Thème réglé sur Sombre, Langue réglée sur English, Échelle de l'interface à 100 %, Mise à l'échelle de l'écran avec suivre l'écran coché, et en dessous Astuces sur une carte vide et Texte d'aide](../assets/interface/preferences-scale-and-tips.png)
