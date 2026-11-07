---
source: 15ac2f37b7d278fa849fb066b91734f671f35658
---

# Enregistrement, mode live et hot reload { #saving-live-mode-and-hot-reload }

## Reconstruire à l'enregistrement { #rebuild-on-save }

Gardez la scène avec votre `FuncGodotMap` ouverte dans l'éditeur Godot et enregistrez simplement dans TrenchDot avec
Ctrl+S. Godot reconstruit aussitôt cette scène, tant que son onglet est celui que Godot affiche et que le nœud de carte a
*Auto Rebuild On Save* activé. Une scène dans un autre onglet qui utilise la carte se reconstruit quand vous passez à son
onglet. Fermer puis rouvrir un onglet ou recharger une scène n'y change rien. Enregistrer un
[prefab](../editor/prefab-library.md) reconstruit les cartes de cette scène qui l'utilisent, directement ou par un autre
prefab.

La barre d'état en bas à gauche vous dit ce que Godot a fait. Quand ça a marché, elle doit dire :

![La barre d'état qui dit « Lien live : Godot a reconstruit 1 nœud(s) de carte »](../assets/live/status-rebuilt.png)

Enregistrez la scène dans Godot pour garder le résultat. Si la barre d'état dit autre chose, le message vous dit pourquoi :

![La barre d'état qui dit que room.tscn utilise room.tdm mais n'est pas l'onglet de scène actuel dans Godot, elle est reconstruite quand vous y passez](../assets/live/status-not-current-tab.png)

| La barre d'état dit | Faites ceci |
| --- | --- |
| La scène n'est pas l'onglet de scène actuel dans Godot | Passez à cet onglet, elle se reconstruit alors |
| L'onglet de scène actuel n'a pas de `FuncGodotMap` pour la carte | Ouvrez la scène qui construit la carte, ou activez *Auto Rebuild On Save* sur son nœud de carte |
| Godot n'a aucune scène ouverte | Ouvrez dans Godot la scène avec la carte, et si elle est ouverte, lisez la suite |
| Godot ne tourne pas avec l'addon TrenchDot | Ouvrez le projet dans l'éditeur Godot avec l'addon activé |
| Godot a reconstruit la carte, mais les réglages de carte n'ont pas de définition pour certaines classes, qui sont donc construites comme des nœuds simples | Ajoutez les définitions, voir [Entités qui ne font rien](entities-troubleshooting.md) |
| Godot a reconstruit la carte avec des avertissements, voir le panneau Problèmes | Lisez-les dans le panneau Problèmes, où ils commencent par « Godot : » |

Un seul éditeur Godot peut tenir le port du live link. Un second éditeur avec le même projet prévient dans son panneau
Output que le live link n'a pas pu écouter, et TrenchDot continue de parler au premier, qui n'a peut-être aucune scène
ouverte. Fermez l'éditeur que vous n'utilisez pas et l'autre reprend le lien en quelques secondes. (Note : *Godot > Ouvrir
le projet dans l'éditeur Godot* ramène au premier plan un éditeur qui a déjà le projet au lieu d'en démarrer un second.)

## Mode live { #live-mode }

Le mode live pousse vos modifications dans la scène ouverte dans l'éditeur Godot avant que vous enregistriez, pour que vous
puissiez juger l'éclairage et l'échelle dans le vrai moteur de rendu.

1. Ouvrez la scène dans Godot

    La carte doit avoir été enregistrée une fois dans le projet, et la scène ouverte dans Godot a besoin d'un
    `FuncGodotMap` qui l'utilise.

2. Activez-le

    Cliquez sur le bouton de lien à l'extrémité droite de la barre d'outils, à côté du robot Godot, ou utilisez *Godot >
    Mode live*. Le bouton s'allume, et son infobulle dit ce qu'il attend encore, le cas échéant.

    ![La barre d'outils avec le bouton de lien du mode live activé et son infobulle qui dit que les modifications arrivent dans Godot avant l'enregistrement](../assets/live/toolbar-live-mode-on.png)

3. Modifiez comme d'habitude

    Chaque changement apparaît aussitôt dans Godot.

    ![Des modifications dans TrenchDot qui apparaissent dans l'éditeur Godot avant l'enregistrement](../screenshots/TrenchDot-demo-live-view.gif)

Le mode live demande le live link Godot, qui est activé par défaut. Les deux se trouvent dans *Fichier > Préférences*, où
*Mode live Godot* est le même interrupteur que le bouton de la barre d'outils.

![Les lignes Live Link Godot, Mode live Godot et Rechargement à chaud du jeu en cours dans les Préférences](../assets/live/prefs-live-link.png)

Les glissements déplacent les nœuds pendant que vous glissez, et la plupart des modifications ne reconstruisent que ce
qu'elles ont touché, donc l'aperçu suit. Basculer *Omit From Export* sur un calque et modifier des instances de prefab
reconstruisent plutôt toute la carte, une fois que vous faites une pause.

L'aperçu est proche d'une construction complète mais pas toujours identique. Enregistrer fait toujours une construction
complète, et quitter sans enregistrer renvoie Godot au fichier enregistré, donc rien de ce que vous avez essayé en mode live
ne reste si vous ne l'enregistrez pas.

### Quand le mode live reconstruit toute la carte { #when-live-mode-rebuilds-the-whole-map }

Godot reconstruit toute la carte de lui-même quand la scène ne correspond plus à ce qu'une construction complète donnerait.
Quand le mode live démarre sur une scène, il le fait aussitôt, sinon une fois que vous faites une pause dans l'édition :

| Quand | Pourquoi |
| --- | --- |
| Le mode live démarre sur une scène construite avec d'autres définitions d'entités | La scène a été construite avant que vous installiez le pack d'entités Gameplay ou ajoutiez une définition, donc certaines entités sont des nœuds simples |
| Le mode live démarre sur une scène que des modifications live précédentes ont changée | La scène contient encore des modifications live d'une session précédente que vous n'avez jamais enregistrées |
| Les définitions d'entités changent pendant que le mode live tourne | Vous avez enregistré un FGD ou une définition dans l'Inspecteur de Godot, une classe de script a changé, ou de nouveaux fichiers sont arrivés, comme un pack installé |
| Vous enregistrez un prefab utilisé par la carte | La scène montre encore le prefab tel qu'il était |

Les scènes ouvertes hors du mode live gardent leurs entités telles qu'elles ont été construites, et le panneau Output de
Godot avertit quand l'une d'elles a été construite avec d'autres définitions d'entités. Appuyez sur **Build Map** dans la
scène et enregistrez.

### Entités sans définition { #entities-without-a-definition }

Quand le mode live construit une entité dont Godot n'a pas de définition pour la classe, la barre d'état nomme la classe et
le FGD où l'ajouter :

![La barre d'état qui dit que Godot n'a pas de définition pour crypt_lamp, il affiche un nœud simple, ajoutez la classe au FGD de res://crypt_settings.tres](../assets/live/status-no-definition.png)

L'infobulle du bouton du mode live garde la même note tant que la carte utilise cette classe :

![Le bouton du mode live activé, son infobulle qui dit que les modifications arrivent dans Godot avant l'enregistrement, et que Godot n'a pas de définition pour crypt_lamp](../assets/live/toolbar-live-mode-no-definition.png)

Ajoutez la définition et enregistrez-la, et Godot reconstruit la carte de lui-même. La note disparaît à votre prochain
enregistrement. Voir [Entités qui ne font rien](entities-troubleshooting.md) pour savoir où va une définition.

## Hot reload dans un jeu en cours d'exécution { #hot-reload-in-a-running-game }

Pour voir vos enregistrements dans un jeu déjà lancé :

1. Ajoutez l'autoload

    Dans Godot, allez dans *Project > Project Settings > Globals > Autoload* et ajoutez ce script :

    ```
    res://addons/func_godot/src/trenchdot/runtime/trenchdot_hot_reload.gd
    ```

2. Lancez le jeu depuis l'éditeur Godot

    L'autoload n'écoute que dans les jeux lancés depuis l'éditeur Godot, sauf si vous réglez son
    `enabled_in_exported_builds` sur true.

3. Enregistrez dans TrenchDot

    Le jeu reconstruit les cartes qui utilisent le fichier enregistré et émet `map_reloaded(map)`. Utilisez ce signal pour
    remettre le joueur à un point d'apparition ou réinitialiser l'état du jeu. La barre d'état doit se terminer par « le
    jeu en cours a rechargé 1 carte(s) ».

Il écoute sur le port 7843, qui doit correspondre à *Rechargement à chaud du jeu en cours* dans les préférences de
l'éditeur. TrenchDot ne lui envoie des enregistrements que lorsque le live link est activé.
