---
source: 7e5d26a9fedbc9f43e9edcb92409992b19d24676
---

# Navigation { #navigation }

`TrenchDotNav` précalcule un maillage de navigation à partir d'une carte construite, pour que les ennemis et les PNJ
trouvent leur chemin à travers elle.

```gdscript
map.build()
var region := await TrenchDotNav.bake_region(map)
```

`bake_region` précalcule sur un thread de travail, ajoute un `NavigationRegion3D` nommé `navigation` sous la carte et ne
rend la main que lorsque les chemins qui la traversent fonctionnent. Passez un nœud qui contient plusieurs cartes pour
toutes les précalculer dans un seul maillage. Une reconstruction libère la région, donc précalculez de nouveau après elle.

## Ce qui compte { #what-counts }

Le précalcul lit les collisions de la carte, pas ses maillages, donc tout ce qu'un joueur peut heurter est ce qu'un agent
contourne.

| Dans la carte | Dans le maillage de navigation |
| --- | --- |
| Brushes du monde, `func_detail`, terrain et collisions des props | Oui |
| Brushes clip | Oui, ils bloquent comme des murs |
| Brushes illusionary, triggers et props physiques | Non |
| Portes, boutons, plateformes et trains | Non, donc une porte fermée ne scelle pas son embrasure |
| Corps du groupe `navigation_ignore` | Non |

Cela veut dire que les chemins passent à travers les portes fermées, donc un agent doit ouvrir une porte contre laquelle
il avance.

## Taille de l'agent { #agent-size }

`TrenchDotNav.profile(radius, height, max_climb, max_slope)` vous donne les réglages pour une taille d'agent. Le rayon
éloigne les chemins des murs, la hauteur écarte les plafonds bas, *max_climb* est la plus haute marche que l'agent monte et
*max_slope* la rampe la plus raide. Les tailles sont en mètres, la pente en degrés :

| Argument | Par défaut |
| --- | --- |
| `radius` | 0,3 |
| `height` | 1,8 |
| `max_climb` | 0,3 |
| `max_slope` | 45 |

Le maillage utilise des cellules de 0,1 m et arrondit les tailles sur elles. Passez un profil en second argument, par
exemple pour une créature large et basse :

```gdscript
var region := await TrenchDotNav.bake_region(map, TrenchDotNav.profile(0.5, 1.2))
```

(`bake_region` règle la taille de cellule de la carte de navigation sur celle du précalcul, et les autres régions de cette
carte doivent la partager.)

## Cache { #caching }

Le premier précalcul d'une grande carte prend quelques secondes. Le résultat est mis en cache dans
`user://trenchdot_navigation`, selon les collisions et les réglages, donc le démarrage suivant avec la même carte le charge
en une fraction de seconde. Passez `false` en troisième argument pour précalculer sans le cache.

## Vérifier une carte { #checking-a-map }

`TrenchDotNav.bake(map)` précalcule sur le thread appelant et renvoie le `NavigationMesh`, pratique pour les outils et les
tests. `TrenchDotNav.islands(nav)` liste ses parties non connectées sous forme d'indices de polygones, la plus grande
d'abord.

Une pièce qui devrait être accessible mais se trouve sur un îlot à part a souvent une embrasure bloquée par quelque chose
de bas, ou une marche plus haute que `max_climb`. Les dessus de murs et le sol à l'intérieur des murs épais apparaissent
aussi comme de petits îlots.

> [!WARNING]
> Godot arrête une recherche de chemin après 4096 polygones et renvoie le chemin parcouru jusque-là, ce qui ressemble à un
> endroit inaccessible sur une grande carte. `TrenchDotNav.path(node, from, to)` n'a pas de limite. Pour les agents,
> réglez `path_search_max_polygons` sur 0 sur le `NavigationAgent3D`.

## Le voir dans TrenchDot { #seeing-it-in-trenchdot }

Vous pouvez voir où un agent marche directement dans TrenchDot, sans lancer le jeu.

1. Ouvrez la scène dans Godot

    Enregistrez la carte dans le projet, et ouvrez le projet dans l'éditeur Godot avec une scène dont le `FuncGodotMap`
    construit cette carte.

2. Activez la zone praticable

    Allez dans *Affichage > Zone praticable > Afficher la zone praticable*. La taille de l'agent se trouve juste en dessous,
    et la ligne du bas vous dit quelle surface est praticable et combien d'îlots sont isolés.

    ![Le menu Affichage avec le sous-menu Zone praticable ouvert : Afficher la zone praticable, puis Rayon 0.30 m, Hauteur 1.80 m, Montée max 0.30 m, Pente max 45.00 degrés et Pas encore précalculé](../assets/navigation/menu-walkable-area.png)

Godot précalcule la scène par le [live link](live-link.md) et TrenchDot dessine le résultat dans les vues 3D et 2D. (S'il
ne peut pas précalculer, la barre d'état dit pourquoi, par exemple que la carte doit d'abord être enregistrée dans le
projet.)

Les couleurs signifient :

| Couleur | Signification |
| --- | --- |
| Vert bleuté | La plus grande zone connectée, là où se trouveront la plupart des agents |
| Vermillon | Les îlots qu'un agent sur la zone verte ne peut pas atteindre. Cherchez une marche plus haute que *Montée max*, un passage plus étroit que deux fois le *Rayon* ou une pente plus raide que *Pente max* |

La superposition se précalcule de nouveau un instant après chaque modification. Sans [Mode live](live.md), Godot construit
d'abord la carte telle qu'elle est affichée, comme *Construire dans Godot*.

> [!TIP]
> Les dessus de murs et de toits apparaissent comme de petits îlots vermillon. Cherchez un sol vermillon à côté d'un sol
> vert.

<div class="td-mcp" markdown>

## MCP { #mcp }

Le [serveur MCP](../mcp.md) permet aux agents et aux scripts de piloter l'éditeur. Ils vérifient une carte avec l'outil
`walkability`, qui renvoie les îlots avec leurs surfaces et leurs limites en unités de carte.

</div>
