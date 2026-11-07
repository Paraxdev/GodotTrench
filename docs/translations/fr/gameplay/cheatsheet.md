---
source: 414dc5a3711ba57511bd4ecc18dc19f159af44d7
---

# Aide-mémoire des E/S { #io-cheatsheet }

Un résumé d'une page de [Comment fonctionnent les E/S d'entité](io.md) et de
[Cibles, paramètres et entrées](parameters.md), pour quand vous connaissez l'idée et n'avez besoin que des détails.

## Quand une sortie s'active { #when-an-output-fires }

Le signal s'active, compte dans *Fois*, attend *Délai*, puis appelle l'entrée sur chaque nœud que la cible trouve. S'il
n'en trouve aucun, rien ne se passe, et un targetname ou un chemin de nœud qui ne trouve rien avertit une fois.

## Cibles { #targets }

| Écrivez | Atteint |
| --- | --- |
| `main_door` | Toute entité avec ce targetname, dans la même carte |
| `door_*` | Tout targetname qui commence par `door_` |
| `@enemies` | Tout nœud de ce groupe Godot, n'importe où dans l'arbre |
| `!player` | Le premier nœud du groupe `player` |
| `!activator` | Celui qui a lancé la chaîne, `!caller` veut dire la même chose |
| `!self` | L'entité qui possède la sortie |
| `/root/Game` | Le nœud à ce chemin absolu |

## Paramètres { #parameters }

Une valeur simple est essayée comme int, float, bool, puis `x y z` comme `Vector3`, sinon elle reste une chaîne. Un
`[a, b]` JSON se répartit en plusieurs arguments. `$activator`, `$self`, `$position` et `$caller_name` sont remplis sur
l'entité **qui reçoit**.

Sans paramètre, l'activateur va dans le premier argument objet, et les autres valeurs que la sortie portait remplissent
le reste dans l'ordre.

## Qui garde l'activateur { #who-keeps-the-activator }

| Signaux | Activateur |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Transmis tel quel |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Remplacé par le nœud entre parenthèses |
| `opened`, `timer`, `step_N`, `on_true`, `on_case_N`, `result`, `broken`, `switched`, `map_spawn` | Perdu |

Si la chaîne l'a perdu et que vous avez besoin du joueur, ciblez `!player`.

## Entrées que tout nœud a { #inputs-every-node-has }

`kill`, `show`, `hide`, `enable`, `disable` et `toggle` marchent sur n'importe quel nœud, tant que l'entité n'a pas de
méthode de ce nom. Une entrée qui correspond à une propriété la règle sur le paramètre.

## Portée de logic_script { #logic_script-scope }

Un [`logic_script`](entities/scripting.md) exécute un extrait de GDScript quand il est activé. Dedans, `this` est le
logic_script lui-même, `activator` le nœud qui a lancé la chaîne, `parameter` la valeur passée, `io` les utilitaires
`TrenchDotIO` et `tree` le `SceneTree`. Indentez avec des tabulations.

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

## Rien ne s'est passé ? { #nothing-happened }

Vérifiez le panneau Problèmes, puis le Graphe logique, puis F3 en jeu, puis journalisez `TrenchDotIO.events()`. Voir
[Déboguer le câblage](debugging.md).
