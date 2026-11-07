---
source: 0215c0eddc11970f765e58b4ce44056eebf2815e
---

# Streaming des grandes cartes { #streaming-big-maps }

Les grandes cartes en extérieur peuvent cacher ce qui est loin, pour que la carte coûte ce que le joueur voit au lieu de
tout ce qu'elle contient. La construction range les maillages de la carte en chunks sur une grille, et seuls les chunks
proches de la caméra sont dessinés.

## L'activer { #turning-it-on }

1. Ne sélectionnez rien

    Appuyez sur Échap pour que rien ne soit sélectionné. L'Inspecteur à droite affiche alors *Carte (worldspawn)*.

2. Ajoutez `chunk_streaming`

    Saisissez `chunk_streaming` dans le champ *clé* en bas, `1` dans *valeur*, et cliquez sur **Ajouter**. (S'il est déjà
    listé, activez-le simplement là.)

    ![L'Inspecteur de worldspawn avec chunk_streaming saisi dans le champ de clé, 1 dans le champ de valeur et le bouton Ajouter survolé](../assets/streaming/worldspawn-add-key.png)

    Vous devez le voir listé avec les autres clés de worldspawn :

    ![L'Inspecteur de worldspawn avec chunk_streaming coché](../assets/streaming/worldspawn-chunk-streaming.png)

3. Enregistrez la carte

    La prochaine construction ajoute un `TrenchDotStreamer` à la carte.

(Facultatif) ajoutez `chunk_size` et `load_radius` de la même façon :

| Clé | Par défaut | Signification |
| --- | --- | --- |
| `chunk_size` | 2048 | Taille d'un chunk en unités de carte, 64 m. Des chunks plus petits cachent plus précisément mais coûtent plus à vérifier |
| `load_radius` | 8192 | Distance à la caméra jusqu'où les chunks restent visibles, en unités de carte, 256 m |

Le streaming ne cache que les maillages. Les collisions et les scripts continuent de tourner partout, donc il économise le
coût de rendu, pas la physique ni la logique. Les overlays, le contenu créé à l'exécution et tout ce qui est sur une porte
ou un train mobile ne sont jamais cachés.

Le streamer suit la caméra actuelle du viewport, ou un `info_player_start` tant qu'il n'y a pas de caméra. Pour le fixer à
une caméra, réglez son `camera_path` depuis le code après la construction, puisqu'une reconstruction remplace le streamer.
Il ne tourne que dans le jeu, donc l'éditeur Godot montre toujours la carte entière.

Quand des chunks apparaissent et disparaissent, le streamer émet `area_loaded(key, bounds)` et `area_unloaded(key,
bounds)`, ce qui vous permet de streamer votre propre contenu à côté :

```gdscript
streamer.area_loaded.connect(func(key, bounds): spawn_wildlife(bounds))
```

## Streamer des sections { #streaming-sections }

Vous voulez jouer plusieurs cartes comme un seul monde sans écran de chargement ? Découpez-le en sections et reliez-les par
des salles de jonction. Une seule section est dans l'arbre de scène à la fois. La suivante se charge en arrière-plan
pendant que le joueur explore, et s'échange pendant qu'il traverse une jonction, alignée pour que la salle autour de lui
reste exactement la même. Cela demande le [pack d'entités Gameplay](../gameplay/entities/README.md#installing-and-customizing).

1. Construisez une salle de jonction

    Un couloir avec un coude de chaque côté, pour qu'aucune extrémité ne soit visible depuis son milieu. Placez un
    `info_landmark` au milieu, et deux volumes `func_section_stream`, un pour chaque moitié, qui se rejoignent au milieu.
    Ajoutez-la à la [bibliothèque de prefabs](../editor/prefab-library.md) pour que chaque copie soit identique.

2. Collez-la là où deux cartes se rejoignent

    Dans chaque carte qui en rejoint une autre, collez la jonction à l'endroit où les deux se rejoignent et nommez son
    repère, par exemple `lm_hall_cellar`. Réglez le `section` de chaque moitié sur la section de ce côté et `landmark` sur le
    nom du repère. Les deux cartes reçoivent la même copie avec les mêmes noms, tournée dans le même sens.

3. Précalculez l'éclairage dans la carte de chaque section

    SDFGI ne remarque la géométrie qu'un échange apporte qu'une fois ses cascades défilées, donc une section échangée
    resterait non éclairée. L'éclairage précalculé arrive avec la section.

4. Faites une scène par section

    Une scène par section avec sa carte construite, de la façon habituelle.

5. Ajoutez le streamer

    Ajoutez un nœud `TDSectionStreamer` à la scène de votre monde et réglez son `first_section` et son joueur, voir
    [TDSectionStreamer](../gameplay/entities/sections.md#tdsectionstreamer).

> [!TIP]
> Gardez la position d'un repère à un multiple de 480 unités de carte, ou de toute taille sur laquelle toutes les textures
> de la jonction se répètent, pour que les textures s'alignent de la même façon dans les deux copies.
