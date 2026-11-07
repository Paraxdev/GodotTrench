---
source: 77812fdfe3757a8be2d8e1be3d7fbd73a1ff678a
---

# Relire les modifications { #reviewing-changes }

Gardez vos cartes dans git et vous pouvez voir ce qui a changé entre deux versions, comme vous relisez du code. Pour les
changements de la session en cours, le [panneau Historique](organizing.md#history) liste chaque modification et permet
d'annuler jusqu'à n'importe laquelle.

## Les cartes dans git { #maps-in-git }

Les fichiers `.tdm` sont binaires, mais git peut quand même vous montrer des diffs lisibles. Il demande à l'éditeur
d'afficher chaque version en JSON et compare ces textes. Vous ne le configurez qu'une fois :

1. Ajoutez cette ligne à `.gitattributes` dans votre projet :

    ```
    *.tdm binary diff=tdm
    ```

2. Dites à git comment afficher une carte, une fois par clone :

    ```sh
    git config diff.tdm.textconv "trenchdot --dump"
    ```

3. (Facultatif) gardez les sauvegardes et les sauvegardes automatiques hors du dépôt avec ces lignes dans `.gitignore` :

    ```
    *.tdm.bak
    *.tdm.autosave
    ```

`git diff` et `git log -p` doivent maintenant afficher les changements de carte en JSON, au lieu de seulement dire que les
fichiers binaires diffèrent.

> [!WARNING]
> `trenchdot` doit se trouver dans votre `PATH`, sinon `git diff` échoue sur les cartes.

Pour obtenir ce JSON dans un fichier, `trenchdot --to-json map.tdm map.json` l'écrit, et `trenchdot --to-tdm map.json
map.tdm` le retransforme en carte binaire. Voir [Ligne de commande](../command-line.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

Les outils de relecture MCP vous montrent ce qu'un agent IA a fait à votre carte.

### Modifications faites par un agent IA { #edits-made-by-an-ai-agent }

Quand un agent modifie votre carte via [MCP](../mcp.md), chaque appel d'outil est une étape d'annulation nommée `MCP: ...`
dans le [panneau Historique](organizing.md#history). Un `run_script` entier est lui aussi une seule étape, comme
`MCP: Add crates, 6 steps`. Annulez cette étape pour reprendre le travail de l'agent.

![Le panneau Historique avec MCP: Add crates, 6 steps en haut, au-dessus de modifications faites à la main comme Renommer, Grouper et Évider](../assets/reviewing/history-mcp-script.png)

| Pour voir | Demandez à l'agent |
| --- | --- |
| Ce qui a changé depuis votre dernier enregistrement | `changes_since`, qui liste les nœuds ajoutés, supprimés, déplacés ou autrement modifiés |
| Ce que son dernier appel ou script a changé | `changes_since` avec `undo_steps: 1` |
| Ce qui a changé depuis un commit git | `changes_since` avec `file` réglé sur cette version, écrite avec `git show HEAD:maps/level.tdm > old.tdm` |

> [!TIP]
> Demandez à l'agent de mettre un travail en plusieurs étapes dans un seul `run_script` avec un `label`, pour que tout le
> travail s'annule en une seule étape sous un nom que vous reconnaissez.

</div>
