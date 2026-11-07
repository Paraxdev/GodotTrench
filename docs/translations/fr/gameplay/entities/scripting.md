---
source: 2bf109094dcc42ebb11cf6c736e210c0e4e79b02
---

# Scripts et appels { #scripts-and-calls }

> [!NOTE]
> Ces entités viennent avec le pack d'entités Gameplay, voyez
> [Installer et personnaliser](README.md#installing-and-customizing).

Des entités point pour ce que le simple câblage ne peut pas atteindre : l'état du jeu, le code de votre propre jeu, les
animations, et comprendre pourquoi une chaîne ne se lance pas.

## logic_script { #logic_script }

Exécute un bout de GDScript quand il est lancé, pour un peu de logique qu'aucune entité intégrée n'offre, comme lire la
santé du joueur, sans écrire d'entité personnalisée.

* **Entrées :** `run(activator)`, `run_with(parameter)`
* **Sorties :** `ran(result)`, avec la valeur de retour du bout de code
* **Clés :** `source` vide, `expression` vide (une ligne, utilisée quand `source` est vide)

Le bout de code voit `this` (l'entité elle-même), `activator` (celui qui a lancé la chaîne), `parameter` (la valeur
entrante), `io` (les aides `TrenchDotIO`, par exemple pour trouver des entités par leur nom) et `tree` (le `SceneTree`).
`run_with` a toujours un `activator` nul. Une erreur de compilation est consignée comme avertissement au chargement de la
carte. Ce bout de code blesse le joueur :

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

> [!WARNING]
> Indentez les lignes imbriquées avec des tabulations. Les espaces ne compilent pas et donnent une erreur d'indentation
> mixte.

> [!WARNING]
> Le bout de code est du code qui voyage dans la carte. Il s'exécute chaque fois que le jeu tourne, avec les mêmes droits
> que les scripts du projet lui-même, donc traitez une carte téléchargée qui contient du code comme n'importe quel
> programme téléchargé : elle peut tout faire sur votre ordinateur. Le panneau Problèmes avertit pour chaque
> `logic_script` qui contient du code, chaque sortie qui peut en transmettre et chaque Prefab qui a l'un ou l'autre,
> lisez ces avertissements avant de jouer la carte.

## logic_call { #logic_call }

Appelle une méthode sur n'importe quelle cible avec des arguments fixes, pour atteindre le code de votre jeu, comme
ajouter du score sur un Autoload `Game`. Contrairement à une sortie simple, il avertit quand la cible ou la méthode
n'existe pas, donc les fautes de frappe apparaissent dans le journal. Les méthodes C# fonctionnent sous leur nom en
PascalCase.

* **Entrées :** `trigger(activator)`, `call_with(parameter)`
* **Sorties :** `called(result)`
* **Clés :** `call_target` `/root/Game`, `method` vide, `arguments` `[]`

`call_target` est un chemin de nœud, un `@group`, un targetname, `!activator` ou `!self`. `arguments` est un tableau JSON,
comme `["key_red", "$activator"]`, et accepte les marqueurs de [Cibles, paramètres et entrées](../parameters.md),
résolus sur le `logic_call` lui-même. `call_with` met son paramètre là où se trouve `"$parameter"`, ou le passe comme
unique argument.

## logic_animate { #logic_animate }

Joue des animations sur le premier `AnimationPlayer` à `target` ou en dessous, pour les personnages et les props qui
portent leurs propres animations. Réglez `target` sur `!activator` pour animer celui qui a lancé la chaîne.

* **Entrées :** `play(name)`, `stop`, `queue(name)`, `seek(time)`
* **Sorties :** `finished(anim)`
* **Clés :** `target` `!self`, `animation` vide (jouée quand `play` n'a pas de nom), `speed` 1.0

## logic_debug { #logic_debug }

Écrit une ligne sur `write` et affiche ses dernières lignes dans une étiquette flottante dans les builds de debug, pour
voir en jeu si une chaîne l'atteint et avec quel paramètre. Avec `trace_all` activé, il consigne chaque sortie du jeu,
un moyen rapide de surveiller toute une chaîne. Voyez [Déboguer le câblage](../debugging.md).

* **Entrées :** `write(parameter)`, `toggle` (l'étiquette), `show`, `hide`
* **Sorties :** `printed(text)`
* **Clés :** `message` `{name} received {parameter} from {activator}` (`{time}` fonctionne aussi), `show_label` 1,
  `trace_all` 0
