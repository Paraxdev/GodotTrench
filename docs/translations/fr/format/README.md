---
source: d3f0019b345d8a4b85cb01b22ccb9fd1cdee21d4
---

# Le format de carte .tdm { #the-tdm-map-format }

Ces pages s'adressent à vous si vous écrivez un outil qui lit ou écrit des fichiers `.tdm`, comme un importeur, un
convertisseur ou une vérification en CI. Vous n'avez besoin de rien de tout cela pour construire des niveaux.

Un fichier `.tdm` contient trois choses : les propriétés du worldspawn, un peu d'état de l'éditeur et un arbre de nœuds
qui porte le contenu proprement dit.

L'éditeur l'enregistre comme un conteneur binaire de blocs compressés avec zstd, si bien que les fichiers restent
petits et qu'un fichier endommagé s'ouvre quand même. Les anciens éditeurs enregistraient à la place un seul document
JSON en UTF-8, et tous les lecteurs chargent les deux formes. Les deux formes contiennent le même arbre, ces pages le
montrent donc sous la forme du JSON qu'affiche `trenchdot --dump`.

| Page | Lisez-la quand vous |
| --- | --- |
| [Structure du conteneur](container.md) | Analysez ou écrivez le fichier binaire. Elle couvre l'en-tête du fichier, les blocs et la manière dont chaque valeur est encodée en octets |
| [Fichiers endommagés](recovery.md) | Voulez savoir ce qui se charge encore depuis un fichier corrompu, et où chaque lecteur signale ce qui a été perdu |
| [Nœuds](nodes.md) | Parcourez l'arbre. Elle explique la forme que tous les nœuds partagent, le fonctionnement des ids, et les nœuds de calque, de groupe, d'entité et d'instance |
| [Géométrie](geometry.md) | Lisez ou construisez des brushes et des meshes, avec leurs faces, leur projection de texture et leurs displacements |
| [Terrain et scatter](terrain-scatter.md) | Gérez les terrains en heightmap et les ensembles de modèles dispersés |
| [Structure JSON](json.md) | Travaillez avec la forme lisible, convertissez entre les deux formes, ou lisez ce que l'éditeur place dans le presse-papiers |

Vous voulez voir du code qui fonctionne ? L'implémentation de référence est
[`format.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/format.rs), avec le conteneur dans
[`binary.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/binary.rs) et
[`variant.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/variant.rs). Côté Godot,
[`tdm_file.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_file.gd) lit le fichier et
[`tdm_parser.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_parser.gd) le construit.

## Niveau supérieur { #top-level }

| Clé | Type | Écrite | Signification |
| --- | --- | --- | --- |
| `format` | chaîne | toujours | Toujours `"trenchdot-map"`. Elle distingue une carte des autres JSON |
| `version` | entier | toujours | Version de la carte, actuellement 1 |
| `properties` | objet de chaîne vers chaîne | si non vide | Les clés du worldspawn, triées par nom |
| `editor` | objet | si tout n'est pas par défaut | État de la vue de l'éditeur. Il n'est jamais exporté vers Godot |
| `textures` | objet | si une texture a des réglages | [Réglages de texture](geometry.md#texture-settings) par nom de matériau |
| `layers` | tableau de nœuds | toujours | Les [nœuds de calque](nodes.md#layer) de la carte, dans l'ordre. Tout le reste est suspendu en dessous |
| `lightmap` | objet | cartes JSON avec un précalcul | L'éclairage précalculé, qu'une carte binaire garde dans son [bloc LMAP](container.md#lmap) |

Seuls les nœuds `layer` sont lus dans `layers`, et une carte qui n'en a aucun reçoit un calque `Default` au chargement.

Les valeurs du worldspawn sont toujours des chaînes, comme dans un fichier `.map`. Quand Godot construit la carte,
`TrenchDotEnvironment` transforme ces clés en ciel, brouillard et soleil :

| Clés | Ce qu'elles contrôlent |
| --- | --- |
| `sun_angles`, `sun_color`, `sun_energy` | La lumière directionnelle du soleil : où elle pointe, sa couleur et son intensité |
| `ambient_color`, `ambient_energy` | La lumière qui atteint les surfaces que le soleil n'éclaire pas, pour que les ombres ne soient pas noires |
| `sky_top_color`, `sky_horizon_color`, `sky_ground_color`, `sky_energy` | Le dégradé du ciel procédural et sa luminosité |
| `sky_panorama` | Une image `res://` qui remplace le ciel procédural dans Godot. L'aperçu de l'éditeur et son précalcul continuent d'utiliser les couleurs du ciel |
| `fog_color`, `fog_density` | Le brouillard de distance |
| `glow_intensity`, `ssr` | Le bloom sur les surfaces lumineuses, et les réflexions en espace écran quand `ssr` vaut `1` |

Si votre jeu apporte son propre `WorldEnvironment`, réglez `environment` sur `sun_only` et seul le soleil est construit.
`0` ou `none` ne construit rien, voir [environnement et soleil](../godot/building.md#environment-and-sun).
`sky_source` retient quelle texture les faces de ciel d'une carte importée avaient.

L'objet `editor` est un état de vue qui n'a d'importance que dans l'éditeur :

| Clé | Type | Par défaut | Signification |
| --- | --- | --- | --- |
| `cameras` | objet, clés `"1"` à `"9"` | vide | Positions de caméra enregistrées, chacune `{"position": [x, y, z], "yaw": r, "pitch": r}` avec des angles en radians |
| `cordon` | `{"min": [x, y, z], "max": [x, y, z]}` | aucun | La boîte du cordon en unités de carte, utilisée pour travailler sur une partie d'une grande carte |
| `cordon_enabled` | booléen | `false` | Masque les objets hors du cordon et les laisse de côté dans les exports avec cordon |
| `bake` | `{"quality", "texel_size", "softness", "backend"}` | aucun | Les options avec lesquelles la carte a été [précalculée](../editor/light-baking.md) la dernière fois, pour que le prochain précalcul parte d'elles |
| `graph_frames` | tableau | vide | Cadres du [graphe Logic](../gameplay/logic-graph.md#frames), chacun `{"title", "rect": [x, y, width, height], "color"}` en points de graphe. Un cadre endommagé est ignoré au chargement |

## Versions et compatibilité { #versions-and-compatibility }

`format` et `version` décrivent les données. La version du conteneur, dans l'en-tête du fichier, est distincte, elle ne
change que lorsque la disposition des blocs change.

Voici ce que fait chaque lecteur d'un fichier qu'il ne comprend pas entièrement :

| Situation | Éditeur | Addon Godot |
| --- | --- | --- |
| `format` différent | Refuse le fichier | Erreur de construction |
| `version` plus récente | Refuse le fichier | Erreur de construction demandant une mise à jour de l'addon |
| `version` absente | La lit comme la plus ancienne version | Pareil |
| Clé inconnue | L'ignore, et elle disparaît à l'enregistrement suivant | L'ignore |
| Bloc inconnu | Le garde et le réécrit à l'enregistrement | Le saute |
| Indice de face hors limites | Refuse la carte, en nommant le nœud | Saute ce brush |
| `heights` de displacement de la mauvaise taille | Refuse la carte, en nommant le nœud | Construit la face à plat |

L'éditeur abandonne les clés inconnues quand il enregistre. Si vous ajoutez donc des données qu'un ancien éditeur ne doit
pas perdre, il vous faut soit une hausse de la version de la carte, que les anciens éditeurs refusent d'ouvrir, soit un
bloc à part, qu'ils gardent intact. Voir [Autres blocs](container.md#other-chunks).

## Coordonnées et unités { #coordinates-and-units }

Les positions utilisent les axes de Godot : Y vers le haut, X vers la droite, Z vers l'observateur. La construction
divise par le `inverse_scale_factor` des réglages de carte, 32 par défaut, donc 32 unités de carte font un mètre.

| Grandeur | Unité |
| --- | --- |
| Positions, tailles, hauteurs, espacements, taille de cellule | Unités de carte |
| Angles des entités, des instances et du scatter | Degrés : pitch autour de X, yaw autour de Y, roll autour de Z, appliqués dans l'ordre YXZ comme le `rotation_degrees` de Godot |
| Yaw et pitch des signets de caméra | Radians |
| Décalage UV | Texels |
| Échelle UV | Unités de carte par texel |
| `uvs` d'un mesh | Largeurs de texture, donc 1 vaut une répétition complète de la texture |
