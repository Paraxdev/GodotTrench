---
source: b890aefb21f6d8de673ab4ebc70a71d93fd598c5
---

# Cibles, paramètres et entrées { #targets-parameters-and-inputs }

Une connexion nomme sa cible, son entrée et un paramètre facultatif sous forme de texte brut. Cette page montre comment ce
texte devient les nœuds, la méthode et les valeurs utilisés en jeu.

## Cibles { #targets }

La cible d'une connexion, et celle de clés comme `call_target`, décide quels nœuds reçoivent l'entrée. La plupart du
temps, c'est un targetname que vous avez donné à une autre entité. Les autres formes atteignent des groupes d'éléments et
des nœuds hors de la carte.

| Cible | Atteint | À utiliser pour |
| --- | --- | --- |
| `main_door` | Toute entité avec ce targetname dans la même carte | Le cas normal, une entité nommée ou plusieurs qui partagent un nom |
| `door_*` | Tout targetname qui commence par `door_` | Ouvrir tout un ensemble d'un coup. Seul un `*` final marche |
| `@enemies` | Tout nœud du groupe Godot `enemies`, n'importe où dans l'arbre | Ce qui apparaît à l'exécution ou que le code du jeu place |
| `!player` | Le premier nœud du groupe `player` | Atteindre le joueur quand la chaîne a perdu l'activateur |
| `!activator` | Celui qui a lancé la chaîne, `!caller` veut dire la même chose | Blesser ou téléporter celui qui a touché un trigger |
| `!self` | L'entité qui possède la sortie | Une entité qui agit sur elle-même, comme un bouton qui se désactive |
| `/root/Game` | Un chemin de nœud absolu, il doit commencer par `/` | Les autoloads et le reste du code du jeu hors de la carte |

Les targetnames sont cherchés dans le `FuncGodotMap` le plus proche, donc deux cartes dans une même scène ne peuvent pas
atteindre les noms l'une de l'autre. Les groupes, les chemins de nœud et `!player` marchent dans tout l'arbre, servez-vous
en quand une carte doit parler à une autre.

> [!NOTE]
> Dans Source, `!caller` est l'entité qui a activé la sortie. Ici il veut dire la même chose que `!activator`, utilisez
> `!self` pour l'entité elle-même.

### Changements à l'exécution { #runtime-changes }

Les recherches de targetname sont mises en cache par carte pour la vitesse. Après avoir créé ou renommé des entités
nommées à l'exécution, appelez `TrenchDotIO.invalidate(node)` avec n'importe quel nœud de cette carte pour que la
recherche suivante les voie. Renommer veut dire changer la méta `td_targetname`, le `name` du nœud n'y joue aucun rôle.
Tuer une entité n'exige aucun appel, le cache remarque les nœuds libérés.

Les nœuds créés par un spawner n'ont pas de targetname. Atteignez-les par leur groupe, par exemple `@enemies`.

## Paramètres { #parameters }

Le paramètre est une valeur que vous tapez dans la connexion et passez à l'entrée, comme `25` pour `take_damage` ou une
vitesse pour une porte. Le texte devient le premier type qui lui convient :

| Vous tapez | L'entrée reçoit |
| --- | --- |
| `25` | L'entier 25 |
| `0.5` | Le flottant 0.5 |
| `true` ou `false` | Un booléen |
| `0 64 0` | Un `Vector3`, à partir de trois nombres séparés par des espaces |
| `hello` | La chaîne `"hello"`, quand rien d'autre ne convient |

Un tableau JSON comme `[10, "$activator"]` se répartit en plusieurs arguments, pour les entrées qui en prennent plus d'un.
Les valeurs sont converties vers les types que la méthode déclare, donc `1234` arrive sous forme de texte dans un argument
`String`. Quand la méthode prend un nœud avant tout autre argument, un texte comme `!player` ou un targetname est cherché
et passé comme ce nœud.

Un paramètre peut aussi être l'un de ces marqueurs, remplis quand l'entrée est appelée :

| Marqueur | Devient |
| --- | --- |
| `$activator` | L'activateur, le nœud qui a lancé la chaîne |
| `$self`, `$caller` | L'entité qui reçoit, elle-même |
| `$position` | La position globale de l'entité qui reçoit |
| `$caller_name` | Le targetname de l'entité qui reçoit, sinon son nom de nœud |

Les marqueurs sont remplis sur l'entité **qui reçoit**, pas sur celle qui a activé la sortie. Ils doivent être tout le
paramètre ou tout un élément du tableau, ils ne sont pas remplacés à l'intérieur d'un texte plus long.

## Pas de paramètre { #no-parameter }

Laissez le paramètre vide et l'activateur va dans le premier argument typé comme objet, ou non typé et nommé `activator`,
où qu'il soit. C'est pourquoi `take_damage(amount, source)` le reçoit dans `source`. Les autres valeurs que la sortie
portait remplissent les arguments restants dans l'ordre, donc `changed(value)` relié à `set_value` passe la valeur.

Cela transmet aussi `false`, donc `switched(false)` relié au `add` d'un compteur ajoute 0. Mettez un paramètre quand vous
voulez une valeur fixe.

## Comment une entrée est trouvée { #how-an-input-is-found }

Quand une connexion s'active, le nom de l'entrée est comparé à la cible dans cet ordre :

1. Une méthode de ce nom, ou son écriture en PascalCase ou camelCase, donc un `AddScore` en C# répond à `add_score`.
2. Une propriété de ce nom, réglée sur le paramètre ou sur la valeur que la sortie portait. L'entrée `speed` avec le
    paramètre `4` change la vitesse d'une porte, aucune méthode en plus n'est nécessaire.
3. L'une des entrées intégrées ci-dessous, que tout nœud comprend.

| Entrée | Effet |
| --- | --- |
| `kill` | Libère le nœud, le retire du jeu |
| `show`, `enable` | Le rend visible et lui laisse de nouveau traiter |
| `hide`, `disable` | Le rend invisible et met son traitement en pause |
| `toggle` | Inverse seulement la visibilité, le traitement reste comme il est |

Les méthodes propres à l'entité l'emportent, donc `disable` sur un trigger éteint le trigger au lieu de le cacher. `show`
et `hide` sont les intégrées sauf si le script de l'entité définit les siennes. Toute autre entrée affiche
`[TD I/O] <node> has no input '<name>'`.
