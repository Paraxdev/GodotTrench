---
source: 0b6155b82bf9e0fff2345e05627d7c675a868912
---

# Configuration du projet { #project-setup }

Les réglages par défaut de l'addon suffisent pour vos premières cartes. Dès que vous écrivez vos propres entités ou voulez
un autre dossier de textures, donnez au projet ses propres ressources, parce qu'une mise à jour de l'addon remplace les
réglages par défaut.

## Installer et mettre à jour l'addon { #installing-and-updating-the-addon }

À chaque ouverture d'un projet, TrenchDot vérifie son addon. Quand `addons/func_godot` manque ou a une version différente
de celle de l'éditeur, la fenêtre *Configurer ce projet* s'ouvre une fois, et une note jaune reste en bas à droite de la
barre d'état. Survolez-la pour voir les deux versions, cliquez dessus pour rouvrir la fenêtre.

![La barre d'état avec la note jaune « Addon 0.0.9, éditeur 0.1.0 » survolée et son infobulle](../assets/project-setup/status-addon-mismatch.png)

Vous pouvez aussi ouvrir la fenêtre à tout moment avec *Godot > Installer ou mettre à jour l'addon…*.

![Le menu Godot avec Installer ou mettre à jour l'addon… survolé](../assets/project-setup/menu-install-update-addon.png)

Elle ne propose que l'étape dont votre projet a besoin ensuite :

| Le projet a | La fenêtre propose |
| --- | --- |
| Pas d'addon | **Installer l'addon**, puis **Activer le plugin** |
| Un addon plus ancien | **Mettre l'addon à jour en** la version de l'éditeur |
| Un addon plus récent, ou sans version | **Remplacer l'addon par** la version de l'éditeur |
| L'addon, non activé | **Activer le plugin**, qui l'ajoute à `[editor_plugins]` dans `project.godot` et ne change rien d'autre |
| L'addon en checkout git ou en sous-module | Rien. Mettez-le à jour avec git, l'éditeur ne le remplace jamais |

![La fenêtre Configurer ce projet qui dit que l'addon 0.0.9 est plus ancien que l'éditeur, avec le bouton Mettre l'addon à jour en 0.1.0](../assets/project-setup/setup-update-addon.png)

> [!WARNING]
> Fermez Godot avant de mettre à jour. Il continue d'exécuter l'ancien addon jusqu'à son redémarrage, donc le bouton reste
> grisé tant que le live link voit Godot avec le projet ouvert. Activez aussi le plugin avec Godot fermé, sinon Godot
> réécrit sa propre copie de `project.godot`. Le cocher dans Godot sous *Project > Project Settings > Plugins* marche
> aussi.

L'ancien addon n'est pas supprimé, il est déplacé dans `.trenchdot/addon-backups/func_godot-<version>` de votre projet.
Godot ignore les dossiers qui commencent par un point, donc il n'importe pas la sauvegarde et ne la liste pas comme un
second plugin. Supprimez-la une fois que le nouvel addon marche. (Note : le téléchargement est d'abord décompressé et
seulement ensuite mis en place, donc une mise à jour échouée ou annulée laisse l'ancien addon tel qu'il était.)

Si le téléchargement échoue, la fenêtre renvoie vers les [versions](https://github.com/Paraxdev/TrenchDot/releases) et
[itch.io](https://paraxdev.itch.io/trenchdot), qui proposent le même `func_godot-trenchdot-addon.zip`. Extrayez-le dans le
dossier du projet pour obtenir `addons/func_godot`, et remplacez l'ancien dossier au lieu de décompresser par-dessus.

## Entités de base et pack d'entités Gameplay { #core-entities-and-the-gameplay-entities-pack }

Chaque projet reçoit un petit noyau de l'addon : worldspawn, `func_detail`, `func_illusionary`, `func_geo`, `light`,
`light_spot`, `light_probe`, `env_reflection_probe`, `env_voxel_gi` et `info_player_start`.

Les portes, boutons, ascenseurs, trains, triggers, la logique, les spawners, les téléporteurs, les points de passage, les
props, les effets et le reste de la [référence des entités](../gameplay/entities/README.md) viennent avec le pack d'entités
Gameplay. Vous l'installez quand vous le voulez, et il est copié dans `res://trenchdot/entities` :

| Chemin | Contient |
| --- | --- |
| `gameplay_fgd.tres` | Le fichier FGD du pack, qui liste chaque définition |
| `definitions/` | Une définition par entité : ses clés, entrées, sorties et gizmos d'éditeur |
| `scripts/` | Le GDScript que chaque entité exécute |
| `pack.json` | Ce qui a été installé, pour que les mises à jour distinguent vos modifications. Laissez-le tel quel |

Les copies appartiennent à votre projet. Changez un script ou une définition, supprimez les entités que vous n'utilisez pas,
ou ajoutez vos propres définitions à `gameplay_fgd.tres`. L'addon ne définit jamais lui-même ces classnames ni ces classes
de script, donc rien de ce que vous faites là n'entre en conflit avec lui, et une mise à jour de l'addon ne l'annule pas.

### Installer et mettre à jour le pack { #installing-and-updating-the-pack }

1. Installez le pack

    Cliquez sur **Installer les entités Gameplay** dans la fenêtre *Configurer ce projet*, sous l'addon.

    ![La fenêtre Configurer ce projet avec la section des entités Gameplay et son bouton Installer les entités Gameplay](../assets/project-setup/setup-gameplay-entities.png)

    Tant que le projet n'a que le noyau, le même bouton se trouve en haut du menu Gameplay et dans le panneau Entités.

    ![Le menu Gameplay avec Installer les entités Gameplay survolé et son infobulle](../assets/project-setup/menu-install-gameplay-entities.png)

2. Vérifiez que ça a marché

    La fenêtre doit maintenant dire « Installé dans res://trenchdot/entities » en vert, et l'éditeur propose aussitôt les
    nouvelles entités.

    ![La section des entités Gameplay après l'installation, qui dit qu'elle a ajouté 86 fichiers à res://trenchdot/entities et de redémarrer Godot s'il a le projet ouvert](../assets/project-setup/setup-gameplay-installed.png)

    Quand un éditeur Godot a le projet ouvert, TrenchDot lui demande de prendre en compte les nouveaux fichiers, et la
    fenêtre le dit. Une scène en [mode live](live.md#when-live-mode-rebuilds-the-whole-map) se reconstruit alors toute
    seule. Appuyez sur **Build Map** dans les autres scènes qui utilisent vos cartes et enregistrez-les, puisqu'elles ont été
    construites sans le pack. Quand Godot ne tourne pas, ou tourne avec un ancien addon, la fenêtre dit de redémarrer Godot
    s'il a le projet ouvert, puis de construire et d'enregistrer ces scènes.

3. (Facultatif) Mettez-le à jour plus tard

    Quand une mise à jour de l'addon apporte un pack plus récent, la fenêtre propose **Mettre à jour les entités Gameplay**,
    et le menu Gameplay l'affiche avec le nombre de fichiers modifiés. Rien n'est mis à jour tant que vous ne cliquez pas.
    La mise à jour ajoute les nouveaux fichiers et remplace ceux que vous n'avez pas touchés. Les fichiers que vous avez
    modifiés restent tels quels et l'éditeur les nomme, et les fichiers que vous avez supprimés restent supprimés.

Les anciennes cartes faites avant que les entités quittent l'addon continuent de marcher. Quand les cartes d'un projet
utilisent des entités du pack que rien dans le projet ne définit, le pack s'installe de lui-même, à la fois quand l'éditeur
ouvre le projet et quand Godot démarre avec l'addon à jour. Un fichier `.tdm` garde chaque entité dans les deux cas, avec ou
sans définition.

> [!NOTE]
> Le modèle dans l'addon, `addons/func_godot/gameplay_pack`, a un fichier `.gdignore` pour que Godot l'ignore. Gardez ce
> fichier, sinon Godot trouve chaque classe de script du pack en double une fois installé.

### Quelle définition l'emporte { #which-definition-wins }

Le FGD de base, `res://addons/func_godot/fgd/trenchdot/trenchdot_fgd.tres`, lit tout seul le pack installé, donc chaque FGD
construit sur les réglages par défaut de l'addon reçoit le pack sans avoir à le lister. Quand deux définitions partagent un
classname, la dernière l'emporte, dans cet ordre :

1. Les classes de FuncGodot et le noyau de l'addon
2. Le pack installé dans `res://trenchdot/entities`
3. Les entités de votre propre fichier FGD

Pour changer une entité, modifiez sa copie dans le pack, ou définissez le même classname dans votre propre FGD. Les deux
marchent, mais un classname défini deux fois s'oublie facilement, donc l'éditeur vous prévient : quand le projet s'ouvre, la
barre d'état nomme la définition qui l'emporte, et le panneau Problèmes avertit pour chaque classe définie en double qu'une
carte utilise. Supprimez l'une des deux pour trancher.

## Vos propres ressources { #your-own-resources }

Trois ressources travaillent ensemble. Le fichier FGD liste les entités que vos cartes peuvent utiliser. Les réglages de
carte disent à une construction où sont les textures et quel FGD lire. La config de jeu informe TrenchDot des deux, pour que
l'éditeur propose les mêmes entités et textures pendant que vous éditez.

Le plus rapide est de copier `demo_fgd.tres`, `demo_map_settings.tres` et `demo_game_config.tres` depuis
[`godot/demo`](https://github.com/Paraxdev/TrenchDot/tree/main/godot/demo) dans votre projet et de corriger les chemins. Pour
les créer vous-même, faites ceci dans l'éditeur Godot :

1. Créez un `FuncGodotFGDFile`. Réglez *Base Fgd Files* sur `res://addons/func_godot/fgd/trenchdot_default_fgd.tres`, pour
   garder le noyau et le pack installé, puis ajoutez vos propres entités sous *Entity Definitions*. Sans cette base, votre
   FGD n'en a aucune, et même worldspawn se construit sans collision.
2. Créez un `FuncGodotMapSettings`. Réglez *Entity Fgd* sur le FGD de l'étape 1 et *Base Texture Dir* sur votre dossier de
   textures (par défaut `res://textures`). Une nouvelle ressource de réglages de carte démarre sur
   `trenchdot_default_fgd.tres`, qui a le noyau et le pack mais pas vos propres entités.
3. Créez un `TrenchDotGameConfig`. Réglez son *Fgd File* et ses *Map Settings* sur les deux ressources ci-dessus.
   (Facultatif) réglez *Game Name*, l'éditeur l'affiche en haut à droite de la barre d'outils à côté du dossier du projet.
4. Dans *Project > Project Settings*, faites pointer `func_godot/default_map_settings` vers vos réglages de carte et
   `trenchdot/game_config` vers votre config de jeu. Tout nœud de carte dont le champ *Map Settings* est vide les suit. Un
   nœud ajouté avec un ancien addon y a stocké les réglages par défaut de l'addon, donc videz le champ.

L'éditeur propose les entités du FGD de la config de jeu, mais Godot construit avec le FGD des réglages de carte, donc
gardez les deux identiques. Quand ils diffèrent, Godot avertit dans son panneau Output à chaque export de la config de jeu,
et le panneau Problèmes de TrenchDot nomme chaque entité qui manque au FGD avec lequel Godot construit.

> [!TIP]
> Réglez *Entity Name Property* dans les réglages de carte sur `targetname`, comme le fait la démo. Les nœuds construits
> portent alors le nom de leur targetname, ce qui garde l'arbre de scène lisible.

## La config de jeu { #the-game-config }

TrenchDot apprend vos entités, vos dossiers de textures et votre échelle d'unité par `res://trenchdot_game.json`, que la
config de jeu écrit. L'addon l'exporte de nouveau chaque fois que Godot voit des fichiers changer, et l'éditeur la recharge
en moins d'une seconde. L'export nomme aussi les réglages de carte par défaut et les classes que leur FGD définit, ce qui
permet à l'éditeur d'avertir des entités que Godot construirait sans définition. Pour exporter à la main, utilisez *Project >
Tools > TrenchDot: Export Game Config* dans Godot, ou lancez-le en headless, par exemple en CI :

```sh
godot --headless --path <project> --script res://addons/func_godot/src/trenchdot/export_game_config_cli.gd
```

## Réglages du projet { #project-settings }

La plupart des projets ne changent que le premier et le troisième. Ceux du live link comptent quand un autre programme
utilise déjà le port.

| Réglage | Par défaut | Rôle |
| --- | --- | --- |
| `func_godot/default_map_settings` | les réglages par défaut de l'addon | Les réglages de carte pour chaque carte qui n'a pas les siens |
| `func_godot/default_inverse_scale_factor` | 32 | Unités de carte par mètre |
| `trenchdot/game_config` | la config de l'addon | La config qui écrit `trenchdot_game.json` |
| `trenchdot/auto_export_game_config` | true | Exporter de nouveau la config de jeu quand des fichiers changent |
| `trenchdot/live_link_enabled` | true | Lancer le serveur du live link auquel TrenchDot parle |
| `trenchdot/live_link_port` | 7842 | Doit correspondre à *Live Link Godot* dans les préférences de l'éditeur. Les changements s'appliquent aussitôt. Entre 1024 et 65535, toute autre valeur retombe sur 7842 avec un avertissement |
| `trenchdot/threaded_build` | true | Convertir les brushes et les chunks de terrain sur le WorkerThreadPool, pour que les grandes cartes se construisent plus vite |
| `trenchdot/live_chunk_size` | 16.0 | En mode live, la géométrie de worldspawn est découpée en chunks de cette taille en mètres, pour qu'une modification ne reconstruise que son chunk |
| `trenchdot/csharp_entity_dirs` | `res://` | Où les entités C# sont trouvées à la construction |
