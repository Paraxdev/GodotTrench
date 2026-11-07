---
source: 320dd2516530d568c7e2acda31113f93ead3f05b
---

# Sauvegarde et récupération { #saving-and-recovery }

Appuyez sur Ctrl+S pour enregistrer, ou sur Ctrl+Shift+S pour *Enregistrer sous…*. Les boîtes de dialogue de fichier
s'ouvrent dans le dossier de la carte actuelle, ou dans le projet Godot pour une carte que vous n'avez jamais enregistrée,
puisque l'addon charge les cartes par leur chemin `res://`. *Fichier > Cartes du projet* liste toutes les cartes du projet
si vous voulez simplement en ouvrir une rapidement.

## Sauvegardes et sauvegarde automatique { #backups-and-autosave }

L'éditeur garde deux sortes de copies de secours à côté de votre carte :

| Fichier | Écrit |
| --- | --- |
| `<map>.tdm.bak` | À chaque enregistrement, une copie de la version précédente. Renommez-la en `.tdm` pour revenir d'un enregistrement en arrière |
| `<map>.tdm.autosave` | Toutes les trois minutes, pour chaque onglet qui a des modifications non enregistrées |

Pour changer la fréquence de la sauvegarde automatique, allez dans *Fichier > Préférences* et réglez *Enregistrement auto (minutes, 0 = désactivé)*.

![Le champ Enregistrement auto (minutes, 0 = désactivé) dans les Préférences, réglé sur 3](../assets/saving/prefs-autosave.png)

Les sauvegardes automatiques utilisent une extension que Godot n'importe pas, donc une carte à moitié terminée n'apparaît
jamais dans votre jeu. Enregistrer la carte supprime sa sauvegarde automatique.

## Récupérer une carte { #recovering-a-map }

Si l'éditeur s'est fermé avant que vous ayez enregistré, ouvrez la sauvegarde automatique :

1. Choisissez *Fichier > Ouvrir une carte…*.
2. Choisissez le type de fichier **Sauvegarde auto (récupérer une carte)** et ouvrez le fichier `.tdm.autosave`. La barre
   d'état doit indiquer qu'elle a récupéré la carte depuis sa sauvegarde automatique.
3. Enregistrez. Cela réécrit le vrai fichier `.tdm`.

Les cartes que vous n'avez jamais enregistrées sont sauvegardées automatiquement dans un fichier temporaire à la place.
Vous les trouvez sous *Fichier > Ouvrir récent*, sous *Cartes sans titre (auto-enregistrées)*.

![Fichier > Ouvrir récent avec Aucune carte récente en haut et une liste de cartes sans titre sauvegardées automatiquement en dessous](../assets/saving/open-recent-autosaves.png)

## Fichiers endommagés { #damaged-files }

Un fichier de carte endommagé sur le disque s'ouvre quand même. L'éditeur lit chaque partie intacte, place les objets dont
le calque, le groupe ou l'entité a été perdu dans un calque nommé *Recovered*, et indique dans la barre d'état ce qui n'a
pas pu être lu. Enregistrer conserve l'original endommagé sous `<map>.tdm.bak`, vous pouvez donc toujours le restaurer
depuis le contrôle de version.
