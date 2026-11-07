---
source: 92464837c8b6e8506e85d0270bb18896476cd41b
---

# Structure JSON { #json-layout }

La forme JSON est celle qu'enregistraient les anciens éditeurs. C'est aussi celle qu'emploient `--dump`, un
enregistrement `.json`, le presse-papiers et le live link.

Les lecteurs traitent comme du JSON tout fichier qui ne commence pas par `\x89GTM`, avec ou sans marque d'ordre des
octets UTF-8. Une ancienne carte JSON se charge telle quelle et devient binaire à son prochain enregistrement `.tdm`.

[`json_fmt.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/json_fmt.rs) l'imprime de façon à
garder des diffs lisibles. Il indente de deux espaces, garde les clés dans l'ordre où ces pages les listent, et met les
tableaux et objets courts sur une seule ligne. Chaque face tient sur une ligne, comme un tableau `vertices` d'au plus
64 entrées, si bien que déplacer un brush ne change que les lignes de ce brush.

Votre propre outil n'a pas à copier cette disposition, tout JSON valide au même contenu se charge de la même façon.

## Conversion { #converting }

| Commande | Résultat |
| --- | --- |
| `trenchdot --dump map.tdm` | Affiche la carte en JSON |
| `trenchdot --to-json map.tdm map.json` | Écrit la carte en JSON |
| `trenchdot --to-tdm map.json map.tdm` | Écrit une carte binaire, en refusant un fichier que l'éditeur ne pourrait pas ouvrir |

Les conversions copient le contenu du fichier sans le charger dans l'éditeur, elles gardent donc les clés que l'éditeur
ne connaît pas.

L'éditeur ouvre aussi les cartes `.json`, et *Enregistrer sous* avec un nom en `.json` écrit du JSON. Pour des diffs JSON
de cartes binaires dans git, voir [Les fichiers de carte dans git](../development.md#map-files-in-git).

## Presse-papiers { #clipboard }

Copier et coller utilisent la même forme de nœud dans une autre enveloppe :

```json
{
  "format": "trenchdot-clipboard",
  "version": 1,
  "nodes": [ ... ]
}
```

`nodes` contient les sous-arbres copiés sans leurs parents. Coller exécute les mêmes vérifications que le chargement,
donne un nouvel id à chaque nœud et garde `label`, `hidden` et `locked`. Un calque copié colle ses enfants.

Le live link envoie des nœuds isolés dans la même forme, sans `children`.

## Exemple { #example }

Un extrait raccourci de `godot/demo/maps/scripted_scene.tdm` tel que l'affiche `trenchdot --dump`, avec le brush du sol
et un baril explosif :

```json
{
  "format": "trenchdot-map",
  "version": 1,
  "properties": {
    "ambient_color": "60 66 82",
    "classname": "worldspawn",
    "message": "Scripted Scene",
    "sky_ground_color": "60 62 70",
    "sky_horizon_color": "120 130 150",
    "sky_top_color": "40 54 92",
    "sun_angles": "-40 -55",
    "sun_energy": "0.6"
  },
  "layers": [
    {
      "id": 1,
      "type": "layer",
      "name": "Default",
      "color": "#7e67e6ff",
      "omit_from_export": false,
      "children": [
        {
          "id": 2,
          "type": "brush",
          "vertices": [[320.0,8.0,-320.0],[320.0,8.0,320.0],[320.0,0.0,320.0],[320.0,0.0,-320.0],[-320.0,8.0,320.0],[-320.0,8.0,-320.0],[-320.0,0.0,-320.0],[-320.0,0.0,320.0]],
          "faces": [
            {"indices":[0,1,2,3],"material":"showcase/cobble","uv":{"u_axis":[0.0,0.0,-1.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[4,5,6,7],"material":"showcase/cobble","uv":{"u_axis":[0.0,0.0,1.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[1,0,5,4],"material":"showcase/cobble","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,0.0,1.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[3,2,7,6],"material":"showcase/cobble","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,0.0,1.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[2,1,4,7],"material":"showcase/cobble","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[6,5,0,3],"material":"showcase/cobble","uv":{"u_axis":[-1.0,0.0,0.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}}
          ]
        },
        {
          "id": 17,
          "type": "entity",
          "classname": "prop_physics",
          "origin": [0.0,24.0,176.0],
          "angles": [0.0,0.0,0.0],
          "properties": {
            "explosion_damage": "25",
            "explosion_radius": "256",
            "explosive": "1",
            "health": "5",
            "model": "res://demo/scenes/barrel.tscn",
            "size": "16 24 16",
            "targetname": "barrel"
          },
          "outputs": [{"output":"broken","target":"hp_readout","input":"run"}]
        }
      ]
    }
  ]
}
```

Le sol est une dalle de 640 sur 640 unités et de 8 unités d'épaisseur, dont le dessus est à Y 8. Sa première face,
`[0,1,2,3]`, est le côté +X, enroulée dans le sens antihoraire vu depuis +X.

<div class="td-mcp" markdown>

## MCP { #mcp }

Les outils du [serveur MCP](../mcp.md) lisent et renvoient eux aussi des nœuds sous cette forme JSON.

</div>
