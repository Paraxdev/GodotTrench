---
source: 46b4723cec5ef9109b70a2a54e850458f119c7e9
---

# Lancer Godot depuis l'éditeur { #running-godot-from-the-editor }

L'enregistrement, les constructions et le mode live parlent à l'addon par le live link, donc ils marchent avec tout éditeur
Godot qui a le projet ouvert. TrenchDot n'a besoin de trouver l'exécutable Godot que pour *Godot > Ouvrir le projet dans
l'éditeur Godot*, le robot Godot de la barre d'outils, et *Godot > Lancer le projet* (F5) tant qu'aucun éditeur Godot n'a le
projet ouvert, puisque ceux-là démarrent Godot eux-mêmes.

![Le menu Godot avec Lancer le projet survolé, son raccourci F5 à droite et Ouvrir le projet dans l'éditeur Godot en dessous](../assets/running-godot/menu-run-project.png)

## Lancer le projet { #run-project }

Tant qu'un éditeur Godot a le projet ouvert, *Lancer le projet* demande à cet éditeur d'enregistrer ses scènes ouvertes et de
jouer la scène principale, comme quand vous appuyez sur Play dans Godot. Le jeu a alors tout ce que Godot a construit, y
compris les modifications live. La barre d'état doit dire « Godot a enregistré ses scènes ouvertes et lance la scène
principale ». Si le projet n'a pas de scène principale, elle le dit, réglez-en une dans Godot sous *Project > Project
Settings > Application > Run*.

Sans éditeur Godot, *Lancer le projet* démarre le jeu avec l'exécutable Godot. Il joue alors les scènes telles qu'elles ont
été enregistrées en dernier dans Godot. C'est pareil avec un ancien addon, et la barre d'état ajoute que le jeu peut ne pas
avoir vos dernières modifications de carte. Enregistrez-les dans Godot, ou appuyez sur Play là-bas.

## Trouver l'exécutable { #finding-the-executable }

Allez dans *Fichier > Préférences* et regardez *Exécutable Godot*. Quand TrenchDot en a trouvé un, il affiche « Utilise » et
le chemin sous le champ de saisie.

![Le champ Exécutable Godot dans les Préférences avec la ligne « Utilise » et le chemin en dessous](../assets/running-godot/prefs-godot-using.png)

Quand il affiche « Introuvable » en jaune, cliquez sur le bouton « … » à côté du champ et choisissez l'exécutable Godot sur
votre disque, celui qui commence par « Godot_v ». Faites de même s'il a choisi le mauvais Godot. Tant qu'il n'en trouve
pas, Ouvrir le projet dans l'éditeur Godot reste grisé, tout comme Lancer le projet tant qu'aucun éditeur Godot n'a le
projet ouvert.

![Le champ Exécutable Godot vide avec le bouton … et la ligne jaune Introuvable](../assets/running-godot/prefs-godot-not-found.png)

Sans chemin valide dans les Préférences, l'éditeur prend la première correspondance parmi :

1. La variable d'environnement `GODOT`.
2. `godot` ou `godot4` dans le `PATH`, puis les versions téléchargées comme `Godot_v4.7.2-stable_win64.exe`, la plus récente
   d'abord. Les versions Mono l'emportent quand le dossier du projet contient un `.csproj`, puisqu'un projet C# en a besoin.
   Les versions console sont ignorées.
3. Les dossiers d'installation courants : WinGet, Scoop, Steam et `Programs/Godot` sous Windows, `~/.local/bin` et Steam
   sous Linux, `/Applications/Godot.app` sous macOS.

## Le bouton Godot { #the-godot-button }

Le robot Godot à l'extrémité droite de la barre d'outils montre la connexion. Survolez-le pour voir ce qu'il ferait.

Gris veut dire qu'aucun projet n'est ouvert ou que Godot est introuvable, donc un clic ne fait rien.

![Le robot Godot gris survolé, son infobulle qui dit que Godot est introuvable](../assets/running-godot/toolbar-godot-not-found.png)

Normal veut dire que Godot a été trouvé. Cliquez dessus pour ouvrir le projet dans l'éditeur Godot.

![Le robot Godot normal survolé, son infobulle qui dit Ouvrir le projet dans l'éditeur Godot](../assets/running-godot/toolbar-godot-found.png)

Cyan veut dire qu'un éditeur Godot a ce projet ouvert. Cliquez dessus pour ramener cet éditeur au premier plan.

![Le robot Godot cyan survolé, son infobulle qui dit que Godot 4.7.2-stable (official) a ce projet ouvert, cliquez pour l'afficher](../assets/running-godot/toolbar-godot-open.png)

Une roue tourne à côté pendant que Godot construit. Si l'infobulle dit que le Godot en cours a un addon plus ancien,
mettez-le à jour avec *Godot > Installer ou mettre à jour l'addon…* pour obtenir le mode live.

## Confiance dans le projet { #project-trust }

Godot exécute les scripts d'un projet dès qu'il l'ouvre, le joue ou le construit, et un projet ou un pack de cartes que
vous avez téléchargé peut apporter ses propres scripts. Donc quand vous ouvrez un projet que TrenchDot ne connaît pas
encore et qu'il contient du code, TrenchDot demande d'abord.

TrenchDot commence par chercher du code dans le projet. Pendant ce temps, la barre d'état affiche *Recherche de code* et
rien n'est démarré dans Godot sur le projet, donc un gros projet prend un moment avant qu'il demande ou ouvre.

![La demande « Faites-vous confiance aux auteurs des fichiers de ce projet ? » avec le dossier du projet, ce que le Mode restreint bloque, la case du dossier parent et les boutons Faire confiance au projet et Ouvrir en mode restreint](../assets/running-godot/trust-prompt.png)

Cliquez sur **Faire confiance au projet** pour vos propres projets et ceux de personnes en qui vous avez confiance. Cochez
d'abord *Faire confiance aux auteurs de tous les fichiers du dossier parent* pour faire confiance à tous les projets de ce
dossier, comme celui où vous rangez tous vos jeux. Sans cela, vous ne faites confiance qu'à ce projet, et un projet
décompressé à l'intérieur, comme un pack de cartes avec son propre `project.godot`, n'est pas approuvé avec lui.

Cliquez sur **Ouvrir en mode restreint** quand vous n'êtes pas sûr. Vous pouvez toujours voir et modifier les cartes, mais
TrenchDot ne démarre pas Godot sur le projet, ne construit pas dans Godot et n'envoie pas de modifications live. *Mode
restreint* s'affiche alors en jaune à droite de la barre d'état, cliquez dessus pour répondre de nouveau à la question.

![La pastille jaune Mode restreint dans la barre d'état, son infobulle qui dit que TrenchDot n'exécute pas ce projet dans Godot tant que vous ne lui faites pas confiance](../assets/running-godot/restricted-mode-chip.png)

En Mode restreint, *Lancer le projet*, le robot Godot, *Construire dans Godot* et l'activation du mode live demandent « Cela
exécute ses scripts. Lui faire confiance et continuer ? », et continuent une fois que vous cliquez sur **Faire confiance au
projet**.

Ouvrir une carte qui appartient à un autre projet auquel vous n'avez pas fait confiance garde le projet ouvert, et la
demande ajoute **Garder le projet actuel** à ses boutons. Un `project.godot` dans le dossier du projet ouvert est ignoré,
comme Godot l'ignore. Quand une carte extérieure au projet ouvert exécute son propre GDScript, la ligne d'état le dit et le
panneau Problèmes montre où.

Les projets de *Projets récents* d'avant que TrenchDot demande la confiance comptent comme approuvés, tout comme un projet
avec lequel vous démarrez l'éditeur via `--project`, pour cette session.

> [!NOTE]
> Le Mode restreint ne couvre que ce que TrenchDot démarre. Ouvrir vous-même le projet dans Godot exécute ses scripts comme
> d'habitude.

### Projets sans code { #projects-without-code }

Un projet qui ne contient aucun code s'ouvre sans la question, puisque Godot n'y a rien à exécuter. C'est un dossier vide,
un projet que vous venez de créer, ou un projet avec seulement l'addon de TrenchDot et le pack d'entités Gameplay qu'il a
installé. Ici, le code désigne les scripts, les projets C#, les bibliothèques natives, les scènes et ressources qui
contiennent des scripts, les autoloads, les plugins d'éditeur et les cartes dont le `logic_script` contient du GDScript.
L'addon et le pack ne comptent comme ceux de TrenchDot que tant que chaque fichier est celui que cette version installe,
donc une copie modifiée ou plus ancienne compte comme du code, tout comme un projet trop gros pour être parcouru
rapidement.

TrenchDot regarde de nouveau à chaque ouverture du projet et ne retient rien, donc il demande dès que le projet a gagné des
scripts. Pour garder votre propre projet approuvé, enregistrez une carte dedans avec *Fichier > Enregistrer* tant qu'il ne
contient pas de code. À partir de là, TrenchDot lui fait confiance, même après l'ajout de scripts dans Godot. Enregistrer
une carte dans un projet en Mode restreint n'approuve rien.

### Projets de confiance { #trusted-projects }

Ouvrez *Fichier > Préférences…* et regardez *Projets de confiance* en bas pour voir chaque projet et dossier auquel vous
faites confiance. Cliquez sur **Supprimer** à côté de l'un d'eux pour retirer la confiance. Quand c'est le projet ouvert, il
passe aussitôt en Mode restreint et la pastille jaune s'affiche dans la barre d'état, sauf si le projet ne contient pas de
code.

![La partie Projets de confiance des Préférences qui liste deux projets et un dossier avec tous ses projets, chacun avec un bouton Supprimer, l'infobulle de l'un disant que TrenchDot redemande avant que Godot exécute les scripts ici](../assets/running-godot/trusted-projects.png)

<div class="td-mcp" markdown>

## MCP { #mcp }

Un agent n'a jamais la demande sur son chemin. `open_project` et `get_state` renvoient `restricted`, et les outils qui
feraient exécuter ou construire le projet par Godot échouent avec une erreur tant que vous ne lui faites pas confiance avec
la pastille. `map_file` `open` garde le projet ouvert pour une carte d'un projet non approuvé et renvoie ce projet comme
`other_project`.

`trust` dans les deux résultats dit pourquoi un projet n'est pas en Mode restreint : `stored`, `session` pour `--project`,
ou `no_code`. Les enregistrements qu'un agent fait, avec `map_file`, `run_action` ou dans `run_script`, ne stockent jamais
la confiance, donc un projet sans code dans lequel un agent travaille est redemandé dès qu'il a des scripts.

</div>
