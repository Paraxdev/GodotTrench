---
source: b3ed6fe92fc841e610e206dd64cc4020bf717ee7
---

# Fichiers endommagés { #damaged-files }

Une carte binaire endommagée ne perd que ses blocs endommagés, tout le reste s'ouvre encore. Si vous écrivez un lecteur,
cette page décrit ce qu'il doit faire pour se comporter de la même façon.

Un lecteur parcourt les [blocs](container.md#chunks) à partir de l'octet 12. Un en-tête de bloc valide contient le
marqueur de synchronisation, une vérification d'en-tête qui concorde, une taille brute d'au plus 1 Gio et une étiquette
imprimable. Là où il n'y a pas d'en-tête valide, le lecteur cherche vers l'avant le prochain marqueur de synchronisation
et reprend à partir de là.

Un bloc `HEAD`, `NODE` ou `END` est abandonné quand sa charge utile ne se décompresse pas, échoue à sa somme de
contrôle, a une mauvaise taille ou ne se décode pas.

| Dommage | Résultat |
| --- | --- |
| Un bloc `NODE` | Les nœuds qu'il contient sont perdus |
| Le parent d'un nœud a été perdu | Le nœud passe, avec tout ce qui est en dessous, dans un calque nommé `Recovered` à la fin de la carte. Le calque reçoit un nouvel id au chargement |
| Imbrication de plus de 60 niveaux | Déplacé aussi dans `Recovered`, pour que la lecture ne puisse pas épuiser la pile. Seul un fichier fabriqué exprès a cela |
| Le bloc `HEAD` | Les propriétés du worldspawn et l'état de l'éditeur sont perdus, et la version de la carte vient de `END` |
| `HEAD` et `END` ensemble | Le fichier est refusé |
| Pas de bloc `END` | Le fichier a été coupé. Tout ce qui précède le dernier bloc complet se charge |
| `END` compte plus que ce qui a été lu | Signalé comme le nombre de nœuds, ou de blocs, qui manquent |

Chaque lecteur signale toutes les pertes :

| Lecteur | Rapport |
| --- | --- |
| Éditeur | Ouvre ce qu'il a pu lire, liste les pertes dans la barre d'état et marque la carte comme non enregistrée. Les prefabs endommagés sont aussi signalés dans la barre d'état |
| Addon Godot | Construit ce qu'il a pu lire et affiche un avertissement par perte, précédé de `[TDM]` et du chemin du fichier |
| `trenchdot --dump` | Affiche ce qu'il a pu lire et écrit les pertes sur stderr |

Avant chaque enregistrement, l'éditeur copie l'ancien fichier vers `<map>.tdm.bak`, si bien qu'enregistrer une carte
récupérée garde l'original endommagé à côté. Le nouveau fichier est d'abord écrit dans `<map>.tdm.tmp` puis renommé par
dessus la carte, donc un plantage au milieu d'un enregistrement ne laisse jamais une carte à moitié écrite.
