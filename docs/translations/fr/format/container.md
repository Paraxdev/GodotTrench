---
source: 8142a4423329f307647582ed665e04b916677fd2
---

# Structure du conteneur { #container-layout }

Un `.tdm` binaire est un en-tête de fichier de 12 octets, suivi de blocs mis bout à bout jusqu'à la fin du fichier.
Chaque nombre est en little endian et de largeur fixe.

| Décalage | Taille | Signification |
| --- | --- | --- |
| 0 | 8 | Nombre magique `89 47 54 4D 0D 0A 1A 0A`, soit `\x89GTM\r\n\x1a\n` |
| 8 | 4 | Version du conteneur, u32, actuellement 1 |

Un lecteur refuse une version de conteneur plus récente que la sienne.

Le `\r\n` du nombre magique permet de repérer un fichier passé par une conversion de fins de ligne, dans git par
exemple. Un lecteur signale ce fichier comme altéré au lieu de le lire.

## Blocs { #chunks }

Chaque bloc est un en-tête de 24 octets suivi de sa charge utile. Le marqueur de synchronisation et la vérification
d'en-tête distinguent un vrai en-tête de bloc d'octets au hasard, c'est ainsi qu'un lecteur retrouve le prochain bon
bloc dans un [fichier endommagé](recovery.md).

| Décalage | Taille | Signification |
| --- | --- | --- |
| 0 | 4 | Marqueur de synchronisation `89 47 54 43`, soit `\x89GTC` |
| 4 | 4 | Étiquette, quatre caractères ASCII imprimables comme `NODE` ou `END ` |
| 8 | 4 | Indicateurs, u32. Le bit 0 signifie que la charge utile est une trame zstd |
| 12 | 4 | Taille brute, u32, la taille de la charge utile après décompression |
| 16 | 4 | Taille stockée, u32, le nombre d'octets de charge utile qui suivent l'en-tête |
| 20 | 4 | Vérification d'en-tête, u32 : l'étiquette lue comme un u32, XOR les indicateurs, XOR la taille brute, XOR la taille stockée, XOR `0x47544D43` |

Une charge utile compressée est une trame zstd avec une somme de contrôle du contenu. Une taille brute supérieure à
1 Gio signifie que l'en-tête est endommagé, et un fichier entier a lui aussi des [limites](#limits).

L'éditeur écrit les blocs dans cet ordre :

1. Un bloc `HEAD`.
2. Les blocs `NODE`.
3. Les blocs inconnus qu'il a gardés du fichier chargé.
4. Un bloc `LMAP`, quand la carte a un éclairage précalculé.
5. Le bloc `END`, toujours en dernier.

### HEAD { #head }

L'[objet de niveau supérieur](README.md#top-level) sans `layers`.

### NODE { #node }

Un lot de nœuds entiers :

| Clé | Type | Signification |
| --- | --- | --- |
| `parents` | tableau int64 | L'id du parent de chaque nœud, 0 pour un calque |
| `nodes` | tableau d'objets nœud | Les nœuds, chacun sans `children` |

Les nœuds arrivent dans l'ordre de l'arbre, un parent vient donc toujours avant ses enfants, dans le même bloc ou dans
un bloc antérieur. L'éditeur commence un nouveau bloc dès qu'un lot dépasse 64 Kio avant compression, et ne coupe jamais
un nœud entre deux blocs. Ainsi, un bloc endommagé n'emporte avec lui que ses propres nœuds.

Pour reconstruire l'arbre, ajoutez chaque nœud aux `children` du nœud que désigne l'id de son parent.

### END { #end }

Le bloc `END` permet à un lecteur de vérifier qu'il ne manque rien :

| Clé | Type | Signification |
| --- | --- | --- |
| `version` | entier | La version de la carte, pour que les nœuds se chargent encore quand `HEAD` est endommagé |
| `nodes` | entier | Combien de nœuds contiennent les blocs `NODE` |
| `chunks` | entier | Combien de blocs précèdent `END` |
| `content` | entier | L'id de contenu, voir ci-dessous |

L'id de contenu est un hash FNV-1a de 64 bits des charges utiles brutes de `HEAD` et de `NODE` dans l'ordre du fichier,
réduit à ses 52 bits de poids faible pour qu'il survive comme nombre JSON. Godot le garde avec la scène construite,
si bien qu'une [session live](../godot/live-link.md) peut voir qu'une carte n'a pas changé et sauter la reconstruction.

### LMAP { #lmap }

L'[éclairage précalculé](../editor/light-baking.md), présent seulement quand la carte a un précalcul. C'est un seul
dictionnaire de 18 clés dans un bloc à part, pour que le reste de la carte se charge vite et qu'un ancien éditeur le
garde intact. Comme il est gros, l'éditeur le compresse avec un niveau de zstd plus rapide que les autres blocs. Les
positions et les distances sont en unités de carte.

| Clé | Type | Signification |
| --- | --- | --- |
| `version` | entier | Version de la disposition, actuellement 1. Un lecteur abandonne un précalcul qui en a une plus récente |
| `width`, `height` | entier | Taille de la lightmap en texels |
| `scene` | entier | Empreinte de la géométrie, des lampes et du ciel dont le précalcul est issu, pour savoir quand il est périmé |
| `texel_size` | flottant | Unités de carte par texel |
| `light` | octets | Lumière linéaire, trois demi-flottants little endian (rouge, vert, bleu) par texel, ligne par ligne |
| `shadow` | octets | La part de soleil qui atteint chaque texel, de 0 à 255 |
| `ao` | octets | L'occlusion ambiante par texel, 255 là où rien n'est proche |
| `chart_keys` | tableau int64 | Un id de nœud et un indice de face par chart, la face vaut 0 pour un terrain |
| `chart_rows` | tableau float32 | Huit flottants par chart, deux lignes `a b c d` qui donnent la coordonnée de lightmap de 0 à 1 du chart sous la forme `a*x + b*y + c*z + d` |
| `nodes` | tableau int64 | Des paires d'un id de nœud et de l'empreinte de sa géométrie au moment du précalcul |
| `stale` | tableau int64 | Les nœuds dont la géométrie a changé depuis, ou qui n'existent plus. Les lecteurs sautent leurs charts |
| `fallback` | tableau float32 | La lumière moyenne en rouge, vert et bleu, pour les faces sans chart |
| `probe_points` | tableau float32 | Positions des sondes de lumière, trois flottants chacune |
| `probe_sh` | tableau float32 | Neuf coefficients d'harmoniques sphériques rouge, vert et bleu par sonde, comme Godot les stocke |
| `probe_tetrahedra` | tableau int32 | Quatre indices de sonde par tétraèdre |
| `probe_bsp_planes` | tableau float32 | Un plan par nœud BSP, normale et distance |
| `probe_bsp_children` | tableau int32 | L'enfant au-dessus et en dessous de chaque plan : un autre nœud quand c'est positif, le tétraèdre `t` sous la forme `-t - 1`, rien sous la forme du plus petit int32 |

Les clés de sonde suivent la disposition du `LightmapGIData` de Godot, et un lecteur laisse les sondes de côté quand
leurs tailles ne concordent pas. Un bloc qui ne se charge pas est abandonné, avec un avertissement pour refaire un
précalcul.

Dans une carte JSON, le même dictionnaire se trouve sous une clé `lightmap` de niveau supérieur, avec `light`, `shadow`
et `ao` en texte base64.

### Autres blocs { #other-chunks }

Les lecteurs sautent un bloc dont ils ne connaissent pas l'étiquette. L'éditeur le garde octet pour octet et le réécrit
après ses blocs `NODE` à l'enregistrement.

C'est donc la façon d'ajouter des données que les anciens éditeurs transportent sans les comprendre. Gardez en tête
qu'un ancien éditeur a pu supprimer des nœuds auxquels votre bloc fait référence, un nouveau type de bloc doit donc
rester valide quand ces nœuds ont disparu.

## Encodage des valeurs { #value-encoding }

Chaque charge utile est une valeur dans la sérialisation binaire Variant de Godot, le format de `var_to_bytes`, qui
n'emploie que les types ci-dessous. Chaque valeur commence par un en-tête u32 dont l'octet de poids faible est le type.
Le bit 16 de l'en-tête marque un entier ou un flottant de 64 bits. Le bit 16 sur un autre type, ou tout autre bit,
rend le bloc illisible. C'est aussi le cas d'une valeur placée dans plus de 128 tableaux et dictionnaires imbriqués.

| Type | En-tête | Corps |
| --- | --- | --- |
| null | 0 | Rien |
| booléen | 1 | u32, 0 ou 1 |
| entier | 2 | i32, ou i64 avec le bit 16 activé |
| flottant | 3 | f32, ou f64 avec le bit 16 activé. L'éditeur écrit toujours du f64 |
| chaîne | 4 | longueur en octets u32, puis les octets UTF-8, puis un remplissage de zéros jusqu'à un multiple de 4 |
| dictionnaire | 27 | nombre u32, puis chaque clé (une chaîne) suivie de sa valeur |
| tableau | 28 | nombre u32, puis les valeurs |
| octets | 29 | longueur u32, puis les octets, puis un remplissage de zéros jusqu'à un multiple de 4 |
| tableau int32 | 30 | nombre u32, puis un i32 chacun |
| tableau int64 | 31 | nombre u32, puis un i64 chacun |
| tableau float32 | 32 | nombre u32, puis un f32 chacun |
| tableau float64 | 33 | nombre u32, puis un f64 chacun |

Les lecteurs ignorent le bit 31 du nombre d'éléments d'un dictionnaire ou d'un tableau, que Godot active pour les
conteneurs partagés, mais pas celui du nombre d'éléments d'un tableau compact. Les entiers JSON sont écrits comme des
entiers et les flottants JSON comme des flottants, donc convertir en binaire puis revenir donne le même JSON.

Pour garder des fichiers petits, trois sortes de valeurs reçoivent une forme plus compacte. Un lecteur doit accepter la
forme compacte comme la forme simple :

| Valeur JSON | Stockée comme | Quand |
| --- | --- | --- |
| Tableau de 32 nombres ou plus | Un tableau int32 quand ce sont tous des entiers qui tiennent. Un tableau float32 quand ce sont tous des flottants qu'un flottant 32 bits représente exactement. Un tableau float64 pour les autres flottants | Toujours |
| `heights`, `splat` et `holes` d'un terrain (texte base64) | octets, les données décodées | Quand le base64 est sous forme canonique |
| `instances` d'un scatter | Un seul tableau int32 de colonnes | Quand chaque valeur est un flottant et un multiple entier de son pas |

Les colonnes du scatter contiennent d'abord l'indice d'item de chaque instance, puis tous les x, et ainsi de suite
à travers les huit champs d'une instance. Chaque valeur est stockée comme un nombre entier de pas : 1 pour l'item,
100 par unité pour la position, 10 par degré pour les angles et 1000 pour l'échelle. Diviser redonne le flottant
exact, par exemple `1234 / 100.0` vaut `12.34`.

> [!NOTE]
> `tdm_file.gd` retransforme les colonnes de scatter en tableaux d'instances, mais laisse les octets de terrain en
> `PackedByteArray` et les longs tableaux de nombres en tableaux compacts. Un GDScript qui lit une carte doit les
> accepter à côté des tableaux simples.

## Limites { #limits }

Quelques kilooctets de zstd peuvent se décompresser en gigaoctets, un lecteur donne donc un budget à chaque fichier. Il
prend les blocs `HEAD`, `NODE`, `END` et `LMAP` dans l'ordre du fichier et ne lit pas un bloc qui ne tient pas dans ce
qui reste :

1. Avant la décompression, la taille brute du bloc est retirée d'un budget de 1 Gio.
2. Le bloc décompressé reste en mémoire pendant que ses valeurs sont construites, donc sa taille brute et la mémoire de
   ses valeurs doivent tenir ensemble dans ce qui reste d'un budget de 1200 Mio, à côté de ce que prennent les nœuds
   d'un bloc `NODE` une fois la carte construite à partir d'eux. Seules la mémoire des valeurs et celle des nœuds s'y
   retranchent. Un bloc qui ne tient pas ne prend rien, les blocs suivants peuvent donc encore être lus.

Les blocs avec d'autres étiquettes ne sont jamais décompressés et ne prennent rien. La mémoire se compte ainsi, sans
construire aucune valeur :

| Valeur | Mémoire |
| --- | --- |
| Chaque valeur, clés de dictionnaire et éléments des tableaux compacts compris | 80 octets |
| Une chaîne, en plus | 6 octets par octet, car JSON peut écrire un octet sous la forme `\u0000` |
| Des octets, en plus | `4 * ceil(n / 3) + n` pour `n` octets, leur taille en base64 et la copie dans laquelle un terrain ou un précalcul les décode |
| Un nœud d'un bloc `NODE`, en plus de ses valeurs | 5 Kio pour une entité, 12 Kio pour un ensemble de scatter, 1 Kio pour tout autre nœud |

Les coûts des nœuds sont une borne supérieure de ce que prennent les plus petits nœuds de chaque type dans l'éditeur
une fois la carte ouverte (un groupe ou un brush revient à environ 650 octets et est facturé 1 Kio), si bien qu'un
petit fichier de nombreux minuscules nœuds ne peut pas prendre plus de mémoire que le budget. Les nœuds imbriqués sous
`children` dans un bloc `NODE` sont abandonnés et le lecteur signale combien de nœuds du bloc en contenaient, puisque
l'éditeur ne les écrit jamais là. Les instances de scatter ne sont comptées que par leurs valeurs, ce qui dépasse les
400 octets environ que l'éditeur garde pour une instance construite, car toutes les instances d'un ensemble partagent
leur modèle sur le GPU.

La plus grosse carte de démonstration prend 79 Mio, une instance de scatter de 640 à 720 octets, et la lumière,
l'ombre et l'occlusion d'un précalcul à la plus grande taille d'atlas 299 Mio. L'éditeur refuse d'enregistrer une carte
qui dépasse l'une ou l'autre limite, donc chaque fichier qu'il écrit se lit en entier.
