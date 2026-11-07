---
source: 425ace012cd6d8410cbf64a89bb7dc31275f8980
---

# Depurar el cableado { #debugging-wiring }

La mayoría de los errores de cableado fallan en silencio: la puerta simplemente sigue cerrada. Comprueba en este orden, de la comprobación más rápida a la más a fondo.

> [!TIP]
> Una puerta o un trigger que ni siquiera bloquea al jugador se construyó sin su definición, y eso no es un error de cableado. Mira [Entidades que no hacen nada](../godot/entities-troubleshooting.md).

## 1. El panel Problemas { #1-the-issues-panel }

Abre la pestaña **Problemas**, junto a Outliner e Historial a la izquierda. Señala las salidas dirigidas a un targetname que ninguna entidad tiene, y las entradas que la clase del destino no tiene. Una errata en un targetname es la razón más común de que una puerta siga cerrada.

![El panel Problemas con un error: la salida triggered apunta a la entidad desconocida 'hall_lihgt' en trigger_once, con un botón Corregir](../assets/debugging/issues-panel.png)

Haz clic en un problema para seleccionar su entidad y encuadrarla en las vistas, y luego corrige el nombre en el Inspector.

> [!WARNING]
> **Corregir** en un destino que falta no adivina el nombre correcto, elimina la conexión. Si es una errata, corrige el nombre a mano.

## 2. El grafo lógico { #2-the-logic-graph }

En el [grafo lógico](logic-graph.md) un destino que nadie tiene aparece como un nodo rojo con un cable rojo discontinuo, que dice que ninguna entidad tiene este nombre.

![El grafo lógico: la salida triggered de trigger_once recorre un cable rojo discontinuo hasta un nodo rojo hall_lihgt, ninguna entidad tiene este nombre](../assets/debugging/graph-missing-target.png)

**Simular** activa una salida sobre el papel y resalta toda la cadena, mira [Simular](logic-graph.md#simulate).

## 3. F3 en el juego { #3-f3-in-game }

Añade un nodo `TrenchDotDebugOverlay` a tu escena en Godot, o añádelo como autoload. Ejecuta el juego y pulsa F3: lista las salidas a medida que se activan y dibuja cada volumen `Area3D` con su targetname, así ves si un trigger nunca se activó o se activó sobre nada.

Vuelve a alternarlo después de generar triggers, los volúmenes se recogen cuando se enciende. La tecla es la propiedad `toggle_key` del overlay.

## 4. Registro { #4-logging }

Coloca una entidad `logic_debug` y marca su casilla `trace_all` en el Inspector. Entonces cada salida que se activa se imprime en la salida de Godot.

Desde código, conéctate a `TrenchDotIO.events().fired`, que se emite para cada salida, incluidas aquellas cuyo destino no se resolvió a nada.

```gdscript
TrenchDotIO.events().fired.connect(func(source, output, target, input, parameter):
	print(source.name, ".", output, " -> ", target, ".", input))
```

## Cómo se ve cada error { #what-each-mistake-looks-like }

| Error | Síntoma | Solución |
| --- | --- | --- |
| El destino no se resuelve a nada | `[TD I/O] <node>.<output>: target '<name>' matches no node` la primera vez que se activa | Haz que el destino coincida exactamente con el targetname de la entidad, mayúsculas incluidas |
| El destino no tiene esa entrada | `[TD I/O] <node> has no input '<name>'` en la salida de Godot | Elige una entrada que tenga la clase del destino, mira la [Referencia de entidades](entities/README.md) |
| Una salida que la entidad nunca emite | `[TD I/O] <node> has no signal '<output>'` una vez, cuando se construye el mapa | Elige una salida que tenga la clase, o actívala desde código con `TrenchDotIO.fire_output()` |
| Entidad generada o renombrada en tiempo de ejecución | No se encuentra por su targetname | Llama a `TrenchDotIO.invalidate()`, mira [Cambios en tiempo de ejecución](parameters.md#runtime-changes) |
