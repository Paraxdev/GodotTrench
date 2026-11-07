---
source: 602da12da979c292b377fb3a10cadcc22f1babd5
---

# Entités C# { #c-entities }

Une classe C# peut être une entité plaçable sans définition `.tres`. L'addon lit vos fichiers source : les signaux
deviennent des sorties, les membres exportés des clés et les méthodes marquées des entrées.

Faire tourner des entités C# demande la version .NET de Godot. L'éditeur les liste dans tous les cas, mais Godot ne peut en
construire une que lorsqu'il tourne en version .NET, que le projet C# est compilé et que la classe a `[GlobalClass]`.
Sinon, une entité brush en C# se construit comme un simple `Node3D` sans collision et une entité point comme un nœud vide,
et le panneau Output de Godot dit `Node Class ... not found` avec ces trois points à vérifier.

## 1. Ajouter l'aide { #1-add-the-helper }

Ouvrez le panneau **Référence** (*Affichage > Panneaux > Référence*) et appuyez sur **Ajouter l'aide C# au projet** en
haut à gauche.

![Le panneau Référence avec Ajouter l'aide C# au projet survolé, son infobulle dit Écrit TrenchDot.cs, le pont qu'utilisent les entités C#](../assets/csharp/reference-csharp-helper.png)

Il écrit `res://addons/func_godot/csharp/TrenchDot.cs`, qui contient les attributs. Sans lui, le code de votre entité ne
compile pas.

## 2. Déclarer une entité { #2-declare-an-entity }

```csharp
[GlobalClass]
[TrenchDotEntity("npc_guard", Description = "Patrolling guard",
    Color = "#ff4040ff", Size = "-16 0 -16 16 64 16")]
public partial class NpcGuard : CharacterBody3D
{
    [Signal] public delegate void AlertedEventHandler(Node activator, int level); // output "alerted"
    [Export] public float WalkSpeed { get; set; } = 3.5f;                          // key "walk_speed"
    [TrenchDotInput] public void Alert(Node activator) { }                        // input "alert"
}
```

| Élément | Pourquoi il compte |
| --- | --- |
| `[GlobalClass]` | Obligatoire, la construction de la carte crée le nœud par son nom de classe |
| `Size` | La boîte que l'éditeur dessine pour l'entité, en mins puis maxs en unités de carte |
| `Solid = true` | Fait une entité brush, comme une porte, au lieu d'une entité point |
| `[TrenchDotInput]` | Marque les entrées. Si aucune méthode ne l'a, toute méthode `void` publique avec un nom en majuscule initiale est une entrée |

Les noms sont en snake_case dans la carte et en PascalCase en C#. Pour lire les clés vous-même, implémentez
`_func_godot_apply_properties(Dictionary props)` et servez-vous des lecteurs de l'aide comme
`TrenchDot.Float(props, "walk_speed", 3.5f)`.

> [!TIP]
> L'onglet *Classe C#* du panneau Référence écrit un squelette avec les signaux, exports et entrées de n'importe quelle
> entité, un bon départ pour la vôtre. Il omet la ligne `[TrenchDotEntity]`, ajoutez-la vous-même.

## Où l'addon cherche { #where-the-addon-looks }

L'éditeur et la construction de la carte parcourent chacun des dossiers à la recherche de classes d'entités C#, réglés à
deux endroits :

| Utilisé par | Réglage |
| --- | --- |
| L'éditeur (export de la config de jeu) | *Csharp Source Dirs* sur la ressource de config de jeu |
| La construction de la carte | Réglage de projet `trenchdot/csharp_entity_dirs` |

Les deux valent `res://` par défaut et ignorent `addons`, `bin`, `obj` et les dossiers cachés.

> [!NOTE]
> Une classe C# ne peut pas remplacer une entité de la bibliothèque. Si le FGD définit déjà son classname, le FGD gagne.

## Les E/S depuis C# { #io-from-c }

```csharp
var guide = TrenchDot.FindTargets(this, "guide")[0];
guide.Connect("finished", Callable.From(() => GD.Print("cutscene over")));

foreach (Node barrel in TrenchDot.FindTargets(this, "barrel"))
    TrenchDot.Invoke(barrel, "ignite", "", this);

TrenchDot.FireOutput(this, "alerted", player);
```

L'onglet *Utilisation en C#* du panneau Référence génère ce code pour n'importe quelle entité.

<div class="td-mcp" markdown>

## MCP { #mcp }

Un agent ou un script qui pilote l'éditeur par le [serveur MCP](../mcp.md) obtient le même code généré avec l'outil
`code_reference`.

</div>
