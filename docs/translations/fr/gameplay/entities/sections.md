---
source: 721efa7c5ececa362093af607d2e2c2faa92ad9b
---

# Sections streamées { #streamed-sections }

> [!NOTE]
> Ces entités viennent avec le pack d'entités Gameplay, voyez
> [Installer et personnaliser](README.md#installing-and-customizing).

Des entités pour jouer plusieurs cartes comme un seul monde, sans écran de chargement. Chaque carte est une section, et
les sections se rejoignent dans une salle de liaison dont les deux cartes ont une copie identique.
[Streamer des sections](../../godot/streaming.md#streaming-sections) explique pas à pas comment la mettre en place.

## info_landmark { #info_landmark }

Le point partagé d'une salle de liaison. Mettez-en un au milieu de la salle, avec le même nom dans les deux copies.
Quand la section suivante arrive, elle est placée de façon que son repère tombe exactement là où était celui de
l'ancienne.

* **Clés :** `targetname`

## func_section_stream { #func_section_stream }

Une moitié de salle de liaison, un volume invisible. Tant que le joueur s'y tient, la section qu'il nomme est celle qui
est chargée. Une liaison en demande deux, une par côté, et la ligne où ils se rejoignent est l'endroit où les sections
s'échangent, donc placez-la là où aucun bout de la salle n'est visible, par exemple au milieu d'un couloir en forme de Z.

* **Clés :** `section` (la section de ce côté : le chemin de sa scène, comme `res://levels/pools_2.tscn`, ou un nom
  trouvé dans le `sections_dir` du streamer), `landmark` (le `targetname` de l'`info_landmark` de la salle)

## TDSectionStreamer { #tdsectionstreamer }

Pas une entité mais le nœud qui fait tourner le streaming. Ajoutez-le à votre scène de monde et réglez :

| Propriété | Ce que c'est |
| --- | --- |
| `first_section` | La section où le jeu démarre, un chemin de scène ou un nom |
| `sections_dir` | Le dossier où les noms courts sont cherchés, sous la forme `name.tscn` ou `name.scn` |
| `player_scene` | Le joueur à faire apparaître au `info_player_start` de la première section |
| `player_path` | Ou un joueur déjà présent dans la scène. Sans l'un ni l'autre, le premier nœud du groupe `player` qu'apporte une section est gardé |
| `recenter_distance` | Les mètres dont une boucle de sections peut s'éloigner de l'origine avant que tout le monde soit ramené en arrière, 8000 |

Il émet `section_changed(from, to)` après chaque échange.
