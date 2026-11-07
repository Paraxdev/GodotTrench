---
source: 602da12da979c292b377fb3a10cadcc22f1babd5
---

# C#-Entitäten { #c-entities }

Eine C#-Klasse kann eine platzierbare Entität sein, ohne eine `.tres`-Definition. Das Addon liest deine Quelldateien: Signale werden zu Outputs, exportierte Member zu Schlüsseln und markierte Methoden zu Inputs.

Um C#-Entitäten auszuführen, brauchst du den .NET-Build von Godot. Der Editor listet sie in jedem Fall, aber Godot kann eine nur bauen, wenn es den .NET-Build ausführt, das C#-Projekt gebaut ist und die Klasse `[GlobalClass]` hat. Sonst wird eine C#-Brush-Entität als einfacher `Node3D` ohne Kollision gebaut und eine Punkt-Entität als leerer, und das Output-Panel von Godot sagt `Node Class ... not found` mit diesen drei Dingen zum Prüfen.

## 1. Den Helfer hinzufügen { #1-add-the-helper }

Öffne das Panel **Referenz** (*Ansicht > Panels > Referenz*) und drücke oben links **C#-Helfer zum Projekt hinzufügen**.

![Das Panel Referenz mit C#-Helfer zum Projekt hinzufügen unter dem Mauszeiger, sein Tooltip sagt Schreibt TrenchDot.cs, die Brücke, die C#-Entitäten nutzen](../assets/csharp/reference-csharp-helper.png)

Es schreibt `res://addons/func_godot/csharp/TrenchDot.cs`, das die Attribute enthält. Ohne diese Datei kompiliert der Code deiner Entität nicht.

## 2. Eine Entität deklarieren { #2-declare-an-entity }

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

| Teil | Warum es wichtig ist |
| --- | --- |
| `[GlobalClass]` | Erforderlich, der Karten-Build erzeugt den Node über seinen Klassennamen |
| `Size` | Die Box, die der Editor für die Entität zeichnet, als Minima und dann Maxima in Karteneinheiten |
| `Solid = true` | Macht eine Brush-Entität, wie eine Tür, statt einer Punkt-Entität |
| `[TrenchDotInput]` | Markiert Inputs. Hat keine Methode es, ist jede öffentliche `void`-Methode mit großgeschriebenem Namen ein Input |

Namen sind in der Karte snake_case und in C# PascalCase. Um Schlüssel selbst zu lesen, implementiere `_func_godot_apply_properties(Dictionary props)` und nutze die Leser des Helfers wie `TrenchDot.Float(props, "walk_speed", 3.5f)`.

> [!TIP]
> Der Tab *C#-Klasse* des Panels Referenz schreibt ein Gerüst mit den Signalen, Exports und Inputs jeder Entität, ein schneller Start für deine eigene. Es lässt die Zeile `[TrenchDotEntity]` weg, die fügst du selbst hinzu.

## Wo das Addon sucht { #where-the-addon-looks }

Der Editor und der Karten-Build durchsuchen jeweils Ordner nach C#-Entitätsklassen, festgelegt an zwei Stellen:

| Genutzt von | Einstellung |
| --- | --- |
| Dem Editor (Export der Spielkonfiguration) | *Csharp Source Dirs* an der Ressource der Spielkonfiguration |
| Dem Karten-Build | Projekteinstellung `trenchdot/csharp_entity_dirs` |

Beide haben als Standard `res://` und überspringen `addons`, `bin`, `obj` und versteckte Ordner.

> [!NOTE]
> Eine C#-Klasse kann keine Bibliotheks-Entität ersetzen. Definiert das FGD ihren Classname schon, gewinnt das FGD.

## I/O aus C# { #io-from-c }

```csharp
var guide = TrenchDot.FindTargets(this, "guide")[0];
guide.Connect("finished", Callable.From(() => GD.Print("cutscene over")));

foreach (Node barrel in TrenchDot.FindTargets(this, "barrel"))
    TrenchDot.Invoke(barrel, "ignite", "", this);

TrenchDot.FireOutput(this, "alerted", player);
```

Der Tab *Verwendung in C#* des Panels Referenz erzeugt diesen Code für jede Entität.

<div class="td-mcp" markdown>

## MCP { #mcp }

Ein Agent oder Skript, das den Editor über den [MCP-Server](../mcp.md) steuert, bekommt denselben erzeugten Code vom Tool `code_reference`.

</div>
