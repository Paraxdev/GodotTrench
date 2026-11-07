---
source: 4c71c99f5fabec89f69f0c99554b4f3a60fb4148
---

# Mettre à jour un projet GodotTrench { #updating-a-godottrench-project }

TrenchDot s'appelait autrefois GodotTrench, et les versions bêta se sont brièvement appelées LevelBench. Si vous aviez
configuré un projet ou enregistré des cartes à l'époque, ils continuent de marcher. Vous ne mettez l'addon à jour qu'une
seule fois.

## Étapes { #steps }

1. Démarrez le nouvel éditeur. Au premier démarrage, il copie vos préférences, vos projets de confiance et vos fichiers
   récents depuis l'ancien dossier de réglages. L'ancien dossier reste où il est, donc l'ancien éditeur marche toujours
   aussi.
2. Ouvrez votre projet Godot. La fenêtre de configuration s'ouvre et dit que l'addon du projet est plus ancien que
   l'éditeur, même quand les numéros de version sont identiques. Fermez d'abord Godot, puis appuyez sur **Mettre l'addon à
   jour en** suivi de la version.
3. Ouvrez le projet dans Godot une fois. L'addon déplace ses réglages de projet de `godottrench/` vers `trenchdot/` et
   écrit `trenchdot_game.json` à la place de `godottrench_game.json`.

## Ce qui est conservé { #what-carries-over }

| Vous aviez | Ce qui se passe |
| --- | --- |
| Des cartes `.gtm` | Elles s'ouvrent telles quelles. La ligne d'état dit « room.gtm mis à jour depuis le format GodotTrench, enregistrez pour le conserver ». Enregistrer garde le nom `.gtm`, donc vos scènes retrouvent la carte. Les nouvelles cartes sont enregistrées en `.tdm`. |
| Des variables d'environnement `GODOTTRENCH_` | Elles marchent toujours. Quand les deux noms sont définis, celui en `TRENCHDOT_` l'emporte. |
| Le pack Gameplay dans `res://godottrench/entities` | Il continue de marcher, et vos propres scripts qui étendent ses classes compilent toujours. Si vous installez aussi le nouveau pack, le nouveau l'emporte là où les deux définissent une entité. Laissez l'ancien dossier en place tant que vos scènes l'utilisent. |
| Des packs de nature et des modèles dans `res://godottrench/` | Les cartes qui les placent continuent de marcher. Les préréglages de scatter cherchent dans `res://trenchdot/nature`, donc ajoutez de nouveau le pack de nature pour peindre un nouveau scatter. |
| Des scènes construites par l'ancien addon | Elles se chargent, parce que Godot retrouve les scripts de l'addon déplacé par leur uid. Reconstruire la carte les rafraîchit. |

<div class="td-mcp" markdown>

## Clients MCP { #mcp-clients }

Le serveur MCP s'appelle maintenant `trenchdot`. Renommez-le dans la config de votre client. La variable du jeton est
`TRENCHDOT_MCP_TOKEN`, et l'ancienne `GODOTTRENCH_MCP_TOKEN` marche toujours.

</div>
