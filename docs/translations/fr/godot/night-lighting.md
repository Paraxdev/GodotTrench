---
source: 3df4b9a4462dd78307ebb473881223aa02ea1321
---

# Éclairage de nuit { #night-lighting }

Pour une carte de nuit, vous baissez la lumière globale, pour que les lampes que vous placez fassent le travail.

## Assombrir la carte { #darken-the-map }

Appuyez sur Échap, ou cliquez dans un espace vide d'une vue, pour que rien ne soit sélectionné. L'Inspecteur à droite
affiche alors *Carte (worldspawn)*, les réglages de toute la carte.

Réglez les clés ci-dessous. Si une clé n'est pas encore listée, saisissez-la dans le champ *clé* en bas, sa valeur à côté,
et cliquez sur **Ajouter**.

| Clé | Par défaut | Que faire pour la nuit |
| --- | --- | --- |
| `sun_energy`, `sun_color` | 1.1, `255 245 224` | Baissez l'énergie et choisissez un bleu pâle, et le soleil se lit comme une lune |
| `ambient_energy`, `sky_energy` | 1.0, 1.0 | Baissez les deux, pour que les zones non éclairées et le ciel s'assombrissent |
| `glow_intensity` | 0 | Au-dessus de 0, les lampes et les [matériaux lumineux](materials.md#glowing-materials) font du bloom dans Godot |
| `ssr` | 0 | 1 active les réflexions en espace écran, pour que les rues mouillées et les sols brillants reflètent les lampes |

Voici ce qu'utilise la démo du quartier de nuit :

![L'Inspecteur qui montre Carte (worldspawn) du quartier de nuit : ambient_energy 0.8, glow_intensity 0.9, sky_energy 0.6, un sun_color bleu pâle et sun_energy 0.16](../assets/night-lighting/worldspawn-night.png)

## La vérifier dans l'Aperçu éclairé { #check-it-in-lit-preview }

Appuyez sur F3, ou cliquez sur Aperçu éclairé dans la barre d'outils, pour voir la carte avec ses lumières, son ciel et son
brouillard.

![La barre d'outils avec le bouton Aperçu éclairé survolé et son infobulle, Ombrage : Aperçu éclairé (F3)](../assets/night-lighting/toolbar-lit-preview.png)

Vous devez voir les lampes, les fenêtres éclairées et les enseignes lumineuses se détacher sur un ciel sombre :

![La rue du quartier de nuit dans l'Aperçu éclairé : lampadaires, fenêtres éclairées et l'enseigne lumineuse Corner Shop sous un ciel sombre](../assets/night-lighting/street-lit-preview.jpg)

L'Aperçu éclairé montre toutes les clés ci-dessus sauf la lueur et les réflexions, donc vérifiez-les dans Godot. Pour
obtenir la lumière rebondie des lampes sans la payer à chaque image, [précalculez l'éclairage](../editor/light-baking.md).

## Luminaires { #lamp-fixtures }

Une lampe est une entité `light` ou `light_spot` plus la géométrie de la lampe avec son ampoule lumineuse. Donnez à cette
géométrie un targetname et mettez-le dans la clé `fixture` de la lumière (un `*` final correspond à un préfixe).

La géométrie suit alors la lumière sans câblage : visible tant que la lumière est allumée, cachée tant qu'elle est éteinte.
Réglez `fixture_off` sur `dark` pour la garder visible avec sa lueur éteinte à la place.

![L'Inspecteur d'une lumière avec le targetname alley_lamp, fixture alley_bulb et fixture_off réglé sur dark (émission éteinte)](../assets/night-lighting/light-fixture-keys.png)

Voir [Props, lumières et effets](../gameplay/entities/effects.md) pour les autres clés de lumière.

<div class="td-mcp" markdown>

## MCP { #mcp }

Les cartes de démonstration sont construites par des [scripts MCP](../mcp-scripts.md) qui pilotent l'éditeur par son
[serveur MCP](../mcp.md). `examples/mcp/night_district.json` et `examples/mcp/withered_city.json` sont des cartes de nuit
complètes construites de cette façon.

</div>
