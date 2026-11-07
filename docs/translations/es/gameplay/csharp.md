---
source: 602da12da979c292b377fb3a10cadcc22f1babd5
---

# Entidades en C# { #c-entities }

Una clase C# puede ser una entidad colocable sin una definición `.tres`. El addon lee tus archivos fuente: las señales se convierten en salidas, los miembros exportados en claves y los métodos marcados en entradas.

Ejecutar entidades C# necesita la versión .NET de Godot. El editor las lista de todos modos, pero Godot solo puede construir una cuando ejecuta la versión .NET, el proyecto C# está compilado y la clase tiene `[GlobalClass]`. Si no, una entidad brush en C# se construye como un `Node3D` simple sin colisión y una entidad punto como uno vacío, y el panel Output de Godot dice `Node Class ... not found` con estas tres cosas que comprobar.

## 1. Añade el ayudante { #1-add-the-helper }

Abre el panel **Referencia** (*Ver > Paneles > Referencia*) y pulsa **Añadir ayudante C# al proyecto** arriba a la izquierda.

![El panel Referencia con Añadir ayudante C# al proyecto bajo el cursor, su tooltip dice Escribe TrenchDot.cs, el puente que usan las entidades C#](../assets/csharp/reference-csharp-helper.png)

Escribe `res://addons/func_godot/csharp/TrenchDot.cs`, que contiene los atributos. Sin él, el código de tu entidad no compila.

## 2. Declara una entidad { #2-declare-an-entity }

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

| Parte | Por qué importa |
| --- | --- |
| `[GlobalClass]` | Obligatorio, la construcción del mapa crea el nodo por el nombre de su clase |
| `Size` | La caja que el editor dibuja para la entidad, como mínimos y luego máximos en unidades del mapa |
| `Solid = true` | Crea una entidad brush, como una puerta, en lugar de una entidad punto |
| `[TrenchDotInput]` | Marca las entradas. Si ningún método lo tiene, cada método `void` público con nombre en mayúscula inicial es una entrada |

Los nombres son snake_case en el mapa y PascalCase en C#. Para leer las claves tú mismo, implementa `_func_godot_apply_properties(Dictionary props)` y usa los lectores del ayudante como `TrenchDot.Float(props, "walk_speed", 3.5f)`.

> [!TIP]
> La pestaña *Clase C#* del panel Referencia escribe un esqueleto con las señales, los exports y las entradas de cualquier entidad, un buen punto de partida para la tuya. Deja fuera la línea `[TrenchDotEntity]`, añádela tú.

## Dónde mira el addon { #where-the-addon-looks }

El editor y la construcción del mapa escanean cada uno carpetas en busca de clases de entidad C#, que se fijan en dos sitios:

| Lo usa | Ajuste |
| --- | --- |
| El editor (exportación de la configuración de juego) | *Csharp Source Dirs* en el recurso de configuración de juego |
| La construcción del mapa | Ajuste de proyecto `trenchdot/csharp_entity_dirs` |

Ambos valen `res://` por defecto y saltan `addons`, `bin`, `obj` y las carpetas ocultas.

> [!NOTE]
> Una clase C# no puede reemplazar una entidad de la biblioteca. Si el FGD ya define su classname, gana el FGD.

## I/O desde C# { #io-from-c }

```csharp
var guide = TrenchDot.FindTargets(this, "guide")[0];
guide.Connect("finished", Callable.From(() => GD.Print("cutscene over")));

foreach (Node barrel in TrenchDot.FindTargets(this, "barrel"))
    TrenchDot.Invoke(barrel, "ignite", "", this);

TrenchDot.FireOutput(this, "alerted", player);
```

La pestaña *Uso desde C#* del panel Referencia genera este código para cualquier entidad.

<div class="td-mcp" markdown>

## MCP { #mcp }

Un agente o script que maneja el editor por el [servidor MCP](../mcp.md) obtiene el mismo código generado de la herramienta `code_reference`.

</div>
