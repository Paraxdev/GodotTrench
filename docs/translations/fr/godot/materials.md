---
source: 6f3b7a09923ac7b7e6e7e7d49ae11666c6d47d02
---

# Matériaux { #materials }

Chaque face pointe vers un matériau par son chemin à l'intérieur du *Base Texture Dir* des réglages de carte, sans
l'extension, par exemple `showcase/planks`.

Quand Godot construit la carte, il cherche `showcase/planks.tres` dans ce dossier, ou dans *Base Material Dir* à la place si
vous en avez réglé un, et l'utilise tel quel. Pas de `.tres` ? L'addon fabrique alors un matériau simple à partir de
l'image. TrenchDot prévisualise le même `.tres`, donc les deux côtés se ressemblent.

Vous n'avez besoin d'un `.tres` que quand une simple image ne suffit pas, par exemple pour changer l'échelle, ajouter de la
lueur ou le rendre transparent. Un `.tres` avec seulement un `albedo_color` et sans texture marche aussi, l'éditeur le
dessine dans cette couleur.

Les nouvelles textures et les nouveaux matériaux apparaissent dans l'onglet Matériaux en quelques secondes, sans recharger.

## Taille de texture { #texture-size }

Par défaut, un pixel de texture couvre une unité de carte, donc une texture de 64 pixels se répète tous les deux mètres.
C'est parfait pour le pixel art, mais une photo de 1024 pixels d'un mur de deux mètres s'étirerait sur 32 mètres.

Pour corriger ça, dites au matériau combien d'unités de carte une répétition couvre :

```
[resource]
albedo_texture = ExtResource("1_albedo")
metadata/texture_size = Vector2(64, 64)
```

(Vous pouvez aussi l'ajouter dans l'inspecteur de Godot sous *Metadata*, comme un `Vector2` ou un nombre seul.)

Survolez le matériau dans l'onglet Matériaux pour le vérifier, l'infobulle montre la taille de l'image et combien d'unités
une répétition couvre.

![L'onglet Matériaux avec l'infobulle de night/shop_sign : 1024 x 256 px, 256 x 64 unités par répétition, émissif x3](../assets/materials/tooltip-texture-size.png)

Vous pouvez maintenant échanger une image plus nette sans changer la carte. Les calques de terrain l'ignorent, leur réglage
`tile` fixe déjà la répétition.

## Matériaux lumineux { #glowing-materials }

Un matériau brille quand son fichier règle `emission_enabled`. Avec une couleur d'émission noire, la texture d'émission
brille exactement comme elle a été peinte :

```
[resource]
albedo_texture = ExtResource("1_albedo")
emission_enabled = true
emission = Color(0, 0, 0, 1)
emission_energy_multiplier = 2.5
emission_texture = ExtResource("2_emission")
```

Pas de fichier de matériau ? Placez à côté une image nommée comme l'albédo avec `_emission` ajouté, `night/sign_emission.png`
à côté de `night/sign.png`, et elle devient la carte de lueur.

Les matériaux lumineux reçoivent une étiquette jaune *lueur* dans l'onglet Matériaux :

![Les matériaux du dossier night dans l'onglet Matériaux, où apartment_lit, neon_red et shop_sign portent une étiquette jaune lueur](../assets/materials/glow-tags.png)

Appuyez sur F3 pour l'Aperçu éclairé afin de les voir briller dans la vue 3D. Le bloom autour d'eux ne s'affiche que dans
Godot, voir [Éclairage de nuit](night-lighting.md).

## Transparence { #transparency }

Réglez `transparency` dans le fichier du matériau quand le canal alpha de la texture doit laisser voir à travers. Chaque
mode échange qualité et coût à sa façon :

| `transparency` | Comment ça marche | À utiliser pour | Attention |
| --- | --- | --- | --- |
| `1` blend | Mélange la surface avec ce qu'il y a derrière | Verre, taches douces, fumée | Ne projette pas d'ombres |
| `2` scissor | Les pixels sont soit entièrement affichés, soit découpés | Découpes nettes de près, comme les grilles et les feuilles | Les parties fines disparaissent de loin |
| `3` hash | Tramage des pixels, plus l'alpha est bas moins il y en a d'affichés | Découpes fines vues de loin, comme les clôtures grillagées | Les bords doux deviennent du bruit |

L'infobulle dans l'onglet Matériaux vous dit quel mode un matériau utilise :

![L'onglet Matériaux avec l'infobulle de night/chainlink : alpha-hash, carte normale, double face](../assets/materials/tooltip-alpha-hash.png)
