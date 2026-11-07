---
source: 5b0744accf923762bf8a9e744e70cb250be2483b
---

# Ligne de commande { #command-line }

```sh
trenchdot [options] [map]
```

Sans option, l'éditeur s'ouvre simplement, avec la carte si vous en donnez une. `trenchdot --help` (ou `-h`) liste les
options et quitte.

| Option | Effet |
| --- | --- |
| `--project <dir>` | Charge le projet Godot de `<dir>` ou d'un dossier au-dessus et lui fait confiance pour la session, voir [Confiance dans le projet](godot/running-godot.md#project-trust). Sans elle, l'éditeur charge le dernier projet |
| `--default-prefs` | Démarre avec les préférences par défaut et ne les enregistre jamais. Utile pour les scripts et les tests, et pour vérifier si un problème vient de vos propres réglages |
| `--offscreen` | Ouvre la fenêtre en dehors de tous les écrans, sans focus ni bouton dans la barre des tâches. Elle fait quand même son rendu, donc les scripts et les tests peuvent prendre des captures pendant que vous continuez à travailler |
| `--live-link-port=<port>` | Parle à l'éditeur Godot sur ce port au lieu du port *Live Link Godot* des Préférences, pour cette exécution seulement. Les scripts et les tests passent un port libre, donc leurs enregistrements ne reconstruisent jamais de cartes dans le Godot que vous avez ouvert |
| `--hot-reload-port=<port>` | Fait de même pour le port *Rechargement à chaud du jeu en cours*, sur lequel un jeu en cours écoute pour recharger les cartes enregistrées |
| `--dump <map>` | Affiche une carte en JSON et quitte |
| `--to-json <map> <out>` | Écrit une carte en JSON et quitte |
| `--to-tdm <map> <out>` | Écrit une carte en `.tdm` binaire et quitte |
| `--export-glb <map> <out.glb>` | Écrit la géométrie et les matériaux de la carte en glTF binaire pour Blender et d'autres outils 3D, et quitte |
| `--export-obj <map> <out.obj>` | Écrit la même chose en OBJ, avec un `.mtl` et un dossier de textures à côté, et quitte |

Une option inconnue, ou une option à qui il manque sa valeur, affiche une erreur et quitte avec le code 2, aucune fenêtre
ne s'ouvre. Une conversion ou un export qui échoue affiche pourquoi et quitte avec le code 1.

Les conversions et les exports n'ouvrent jamais de fenêtre, donc ils sont sans danger dans les scripts et les hooks git,
voir [Les fichiers de carte dans git](development.md#map-files-in-git).

Les exports retrouvent le projet Godot de la carte depuis son dossier et utilisent les valeurs par défaut de *Fichier >
Exporter* : toute la carte, sans les objets masqués ni les instances de scatter. La logique d'entité, les triggers et les
scripts sont laissés de côté, voir [Exporter vers Blender et d'autres outils 3D](editor/exporting.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

Ces options règlent le [serveur MCP](mcp.md), pour que des scripts et des agents IA puissent piloter l'éditeur.

| Option | Effet |
| --- | --- |
| `--mcp` | Sert MCP sur stdin et stdout |
| `--mcp-http[=<port>]` | Sert MCP en HTTP sur 127.0.0.1, port 7841 par défaut |
| `--mcp-token=<token>` | Oblige les clients HTTP à envoyer ce jeton, quoi que disent les Préférences. Il l'emporte sur `TRENCHDOT_MCP_TOKEN`, voir [Le jeton HTTP](mcp.md#the-http-token) |

`--default-prefs` va bien avec l'une ou l'autre, pour qu'une session scriptée parte des mêmes réglages à chaque fois. Avec
`TRENCHDOT_DISABLE_MCP=1` défini, aucune des deux options ne démarre de serveur, voir
[Désactiver MCP](mcp.md#turning-mcp-off).

</div>
